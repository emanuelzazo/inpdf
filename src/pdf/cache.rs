//! PDF cache with document and text caching.
//!
//! This module provides a global cache for PDF files that:
//! - Caches parsed `lopdf::Document` objects to avoid re-parsing
//! - Lazily caches extracted text per page for repeated access
//! - Validates cache entries by file mtime to detect stale data
//! - Uses canonical paths to handle symlinks and relative paths

use anyhow::{Context, Result};
use lopdf::Document;
use memmap2::Mmap;
use papaya::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use std::time::SystemTime;

// ==============================================================================
// Cached PDF Entry
// ==============================================================================

/// A cached PDF containing a parsed document and optional per-page text cache.
///
/// The document is wrapped in Arc to allow cheap cloning while sharing
/// the underlying data across all accessors. Text extraction is cached
/// lazily per page to avoid re-extracting text in MCP sessions where
/// multiple operations may access the same pages.
#[derive(Clone)]
pub struct CachedPdf {
    doc: Arc<Document>,
    mtime: SystemTime,
    /// Lazily cached extracted text per page (1-indexed).
    text_cache: Arc<HashMap<u32, Arc<String>>>,
}

impl CachedPdf {
    /// Get a reference to the cached parsed document.
    pub fn document(&self) -> &Arc<Document> {
        &self.doc
    }

    /// Get extracted text for a page, using cache if available.
    ///
    /// The page number is 1-indexed.
    pub fn page_text(&self, page_num: u32) -> Result<Arc<String>, pdf_extract::OutputError> {
        let guard = self.text_cache.pin();
        if let Some(text) = guard.get(&page_num) {
            return Ok(Arc::clone(text));
        }

        // Extract and cache the text.
        let text = extract_text_from_doc_page(&self.doc, page_num)?;
        let text = Arc::new(text);
        guard.insert(page_num, Arc::clone(&text));
        Ok(text)
    }
}

/// Extract text from a single page using pdf-extract's output_doc_page.
fn extract_text_from_doc_page(
    doc: &Document,
    page_num: u32,
) -> Result<String, pdf_extract::OutputError> {
    let mut text = String::new();
    let mut output = pdf_extract::PlainTextOutput::new(&mut text);
    pdf_extract::output_doc_page(doc, &mut output, page_num)?;
    Ok(text)
}

// ==============================================================================
// PDF Cache
// ==============================================================================

/// Global cache for PDF files.
///
/// Uses papaya's concurrent HashMap for lock-free reads and safe concurrent
/// writes. Cache entries are keyed by canonical path to handle symlinks.
pub struct PdfCache {
    cache: HashMap<PathBuf, CachedPdf>,
}

impl PdfCache {
    fn new() -> Self {
        PdfCache {
            cache: HashMap::new(),
        }
    }

    /// Get or load a PDF from the cache.
    ///
    /// If the file is already cached and its mtime matches, returns the
    /// cached entry. Otherwise, loads the file fresh (via mmap) and parses it.
    pub fn get<P: AsRef<Path>>(&self, path: P) -> Result<CachedPdf> {
        let path = path.as_ref();

        // Canonicalize to handle symlinks and relative paths consistently.
        let canonical = std::fs::canonicalize(path)
            .with_context(|| format!("canonicalize path: {}", path.display()))?;

        // Check current file mtime for cache validation.
        let metadata = std::fs::metadata(&canonical)
            .with_context(|| format!("stat PDF file: {}", canonical.display()))?;
        let current_mtime = metadata
            .modified()
            .with_context(|| format!("get mtime for PDF: {}", canonical.display()))?;

        // Try to get from cache first.
        let cache_guard = self.cache.pin();
        if let Some(cached) = cache_guard.get(&canonical) {
            if cached.mtime == current_mtime {
                return Ok(cached.clone());
            }
            // Stale entry - will be replaced below.
        }

        // Load and cache the PDF.
        let cached = load_pdf(&canonical, current_mtime)?;
        cache_guard.insert(canonical, cached.clone());
        Ok(cached)
    }
}

// ==============================================================================
// Global Cache Instance
// ==============================================================================

static PDF_CACHE: OnceLock<PdfCache> = OnceLock::new();

/// Get the global PDF cache instance.
pub fn cache() -> &'static PdfCache {
    PDF_CACHE.get_or_init(PdfCache::new)
}

/// Convenience function to get a cached PDF from the global cache.
pub fn get_cached_pdf<P: AsRef<Path>>(path: P) -> Result<CachedPdf> {
    cache().get(path)
}

// ==============================================================================
// PDF Loading
// ==============================================================================

/// Load a PDF file via memory mapping and parse it.
///
/// We use mmap for efficient loading (the OS handles paging), but we don't
/// store it afterward since lopdf's Document owns all its data independently
/// after parsing.
fn load_pdf(path: &Path, mtime: SystemTime) -> Result<CachedPdf> {
    // Open the file for memory mapping.
    let file = File::open(path).with_context(|| format!("open PDF file: {}", path.display()))?;

    // Create a memory map of the file.
    // SAFETY: The file is opened read-only, and we don't modify the underlying
    // file while the map exists. The map is treated as immutable bytes.
    let mmap = unsafe { Mmap::map(&file) }
        .with_context(|| format!("memory-map PDF file: {}", path.display()))?;

    // Hint to the OS that we'll access the file randomly (PDF parsing jumps
    // around the file structure), so read-ahead would be wasteful.
    #[cfg(unix)]
    {
        // Best-effort advisory; ignore errors since it's just an optimization hint.
        let _ = mmap.advise(memmap2::Advice::Random);
    }

    // Parse the document from the memory-mapped bytes.
    // After this, the Document owns all its data - we don't need the mmap anymore.
    let doc =
        Document::load_mem(&mmap).with_context(|| format!("parse PDF: {}", path.display()))?;
    let doc = Arc::new(doc);

    Ok(CachedPdf {
        doc,
        mtime,
        text_cache: Arc::new(HashMap::new()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Create a minimal but valid PDF.
    ///
    /// The xref offsets must be accurate for lopdf to parse it correctly.
    fn create_minimal_pdf(path: &Path) {
        // Build the PDF incrementally to get correct offsets.
        let header = b"%PDF-1.4\n";
        let obj1 = b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n";
        let obj2 = b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n";
        let obj3 = b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>\nendobj\n";

        let offset1 = header.len();
        let offset2 = offset1 + obj1.len();
        let offset3 = offset2 + obj2.len();
        let xref_offset = offset3 + obj3.len();

        let xref = format!(
            "xref\n0 4\n0000000000 65535 f \n{:010} 00000 n \n{:010} 00000 n \n{:010} 00000 n \n",
            offset1, offset2, offset3
        );
        let trailer = format!(
            "trailer\n<< /Size 4 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            xref_offset
        );

        let mut file = File::create(path).expect("create test PDF");
        file.write_all(header).expect("write header");
        file.write_all(obj1).expect("write obj1");
        file.write_all(obj2).expect("write obj2");
        file.write_all(obj3).expect("write obj3");
        file.write_all(xref.as_bytes()).expect("write xref");
        file.write_all(trailer.as_bytes()).expect("write trailer");
    }

    #[test]
    fn same_file_returns_same_arcs() {
        let dir = std::env::temp_dir().join("inpdf_cache_test");
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let pdf_path = dir.join("test_same_arcs.pdf");

        create_minimal_pdf(&pdf_path);

        // Get the PDF twice.
        let cached1 = get_cached_pdf(&pdf_path).expect("first get");
        let cached2 = get_cached_pdf(&pdf_path).expect("second get");

        // Should return the same Arc instance (pointer equality).
        assert!(
            Arc::ptr_eq(&cached1.doc, &cached2.doc),
            "doc Arcs should be the same"
        );

        // Cleanup.
        std::fs::remove_file(&pdf_path).ok();
    }

    #[test]
    fn modified_file_returns_different_arcs() {
        let dir = std::env::temp_dir().join("inpdf_cache_test");
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let pdf_path = dir.join("test_modified.pdf");

        create_minimal_pdf(&pdf_path);
        let cached1 = get_cached_pdf(&pdf_path).expect("first get");

        // Wait a bit and modify the file to change mtime.
        std::thread::sleep(std::time::Duration::from_millis(10));
        create_minimal_pdf(&pdf_path);

        let cached2 = get_cached_pdf(&pdf_path).expect("second get after modification");

        // Should return different Arc instance since file was modified.
        assert!(
            !Arc::ptr_eq(&cached1.doc, &cached2.doc),
            "doc Arcs should differ after modification"
        );

        // Cleanup.
        std::fs::remove_file(&pdf_path).ok();
    }

    #[test]
    fn symlink_and_real_path_return_same_arcs() {
        let dir = std::env::temp_dir().join("inpdf_cache_test");
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let pdf_path = dir.join("test_symlink_real.pdf");
        let symlink_path = dir.join("test_symlink_link.pdf");

        create_minimal_pdf(&pdf_path);

        // Remove any existing symlink.
        std::fs::remove_file(&symlink_path).ok();

        // Create symlink (Unix only).
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&pdf_path, &symlink_path).expect("create symlink");

            let cached_real = get_cached_pdf(&pdf_path).expect("get via real path");
            let cached_symlink = get_cached_pdf(&symlink_path).expect("get via symlink");

            assert!(
                Arc::ptr_eq(&cached_real.doc, &cached_symlink.doc),
                "doc should be same via symlink"
            );

            std::fs::remove_file(&symlink_path).ok();
        }

        std::fs::remove_file(&pdf_path).ok();
    }
}
