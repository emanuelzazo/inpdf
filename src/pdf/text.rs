use crate::pdf::cache::get_cached_pdf;
use anyhow::{Context, Result};
use std::path::Path;

/// Extract text from specific pages of a PDF.
///
/// Uses the per-page text cache to avoid re-extracting text that was
/// already processed (e.g., by a previous grep operation).
pub fn extract_text_pages<P: AsRef<Path>>(path: P, pages: &[u32]) -> Result<Vec<PageText>> {
    let path = path.as_ref();
    let cached = get_cached_pdf(path).with_context(|| format!("cache PDF: {}", path.display()))?;
    let total_pages = cached.document().get_pages().len() as u32;

    // Validate page numbers.
    for &page in pages {
        if page == 0 || page > total_pages {
            anyhow::bail!("Page {} is out of range (1-{})", page, total_pages);
        }
    }

    let mut results = Vec::new();

    // Extract text for each requested page using the text cache.
    for &page_num in pages {
        let text = cached
            .page_text(page_num)
            .with_context(|| format!("extract text from page {}", page_num))?;
        results.push(PageText {
            page: page_num,
            text: text.to_string(),
        });
    }

    Ok(results)
}

#[derive(Debug, Clone)]
pub struct PageText {
    pub page: u32,
    pub text: String,
}

/// Search for a pattern in PDF text, returning matches with page numbers and context.
///
/// Uses the per-page text cache to avoid re-extracting text. This benefits MCP
/// sessions where multiple grep operations may search the same PDF, or where
/// grep is followed by reading specific pages.
pub fn grep_pdf<P: AsRef<Path>>(
    path: P,
    pattern: &regex::Regex,
    max_results: usize,
) -> Result<Vec<GrepMatch>> {
    let path = path.as_ref();
    let cached = get_cached_pdf(path).with_context(|| format!("cache PDF: {}", path.display()))?;
    let total_pages = cached.document().get_pages().len() as u32;

    let mut matches = Vec::new();

    // Extract and search each page individually for correct page attribution.
    // Text is cached per-page, so subsequent searches or reads are cheap.
    for page_num in 1..=total_pages {
        let page_text = match cached.page_text(page_num) {
            Ok(text) => text,
            Err(_) => continue, // Skip pages that fail to extract
        };

        for (line_idx, line) in page_text.lines().enumerate() {
            let line_number = line_idx as u32 + 1;
            for mat in pattern.find_iter(line) {
                matches.push(GrepMatch {
                    page: page_num,
                    line_number,
                    text: line.to_string(),
                    match_start: mat.start() as u32,
                    match_end: mat.end() as u32,
                });

                if matches.len() >= max_results {
                    return Ok(matches);
                }
            }
        }
    }

    Ok(matches)
}

#[derive(Debug, Clone)]
pub struct GrepMatch {
    pub page: u32,
    pub line_number: u32,
    pub text: String,
    pub match_start: u32,
    pub match_end: u32,
}
