use anyhow::{Context, Result};
use arboard::Clipboard;
use clap::Parser;
use num_format::{Locale, ToFormattedString};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// A struct that represents a single file's data from the yek JSON output.
/// We use serde's `derive` macro to automatically handle deserialization.
#[derive(Deserialize, Debug)]
struct YekFile {
    filename: String,
    content: String,
}

/// Command line arguments for the yek-wrapper tool.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Number of top files to display
    #[arg(long, default_value_t = 9)]
    top_file_count: usize,

    /// Number of top directories to display
    #[arg(long, default_value_t = 6)]
    top_dir_count: usize,
}

/// Approximate token estimation, assuming 4 characters per token.
pub fn estimate_tokens(text: &str) -> usize {
    text.chars().count() / 4
}

fn main() -> Result<()> {
    let args = Args::parse();

    // --- Step 1: Execute `yek --json` and capture its output ---
    let output = Command::new("yek")
        .arg("--json")
        .output()
        .context("Failed to execute `yek --json`. Is 'yek' in your PATH?")?;

    if !output.status.success() {
        // If the command failed, print stderr and exit.
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "`yek --json` failed with status {}:\n{}",
            output.status,
            stderr
        );
    }

    // --- Step 2: Parse the JSON directly from the command's output ---
    let mut files: Vec<YekFile> = serde_json::from_slice(&output.stdout)
        .context("Failed to parse JSON from `yek` output. Is the format correct?")?;

    // --- Step 3: Combine all content and calculate stats ---
    if files.is_empty() {
        println!("✅ No files found in yek output. Nothing to do.");
        return Ok(());
    }

    // Prepare raw combined content for statistics
    let raw_combined_content: String = files.iter().map(|f| f.content.as_str()).collect();

    let file_count = files.len();
    let token_count = estimate_tokens(&raw_combined_content);

    println!("📂 Files: {}", file_count.to_formatted_string(&Locale::en));
    println!(
        "🧮 Estimated tokens: ~{}",
        token_count.to_formatted_string(&Locale::en)
    );

    // Prepare formatted combined content for clipboard
    let mut formatted_combined_content_for_clipboard = String::new();
    for file in &files {
        formatted_combined_content_for_clipboard.push_str(&format!(
            ">>>> {}
",
            file.filename
        ));
        formatted_combined_content_for_clipboard.push_str(&file.content);
        if !file.content.ends_with('\n') {
            formatted_combined_content_for_clipboard.push('\n');
        }
    }

    // --- Step 4: Aggregate and display top N largest directories ---
    let mut dir_sizes: HashMap<String, usize> = HashMap::new();
    for file in &files {
        if let Some(parent) = Path::new(&file.filename).parent() {
            let dir_name = parent.to_string_lossy().into_owned();
            *dir_sizes.entry(dir_name).or_insert(0) += file.content.len();
        }
    }

    let mut sorted_dirs: Vec<(String, usize)> = dir_sizes.into_iter().collect();
    sorted_dirs.sort_by(|a, b| b.1.cmp(&a.1));

    println!("\nLargest directories\n");
    for (dir, size) in sorted_dirs.iter().take(args.top_dir_count) {
        let tokens = estimate_tokens(&String::from_utf8_lossy(&vec![0; *size])); // Approximate tokens for directory size
        println!(
            "- {} (~{} tokens, {} chars)",
            if dir.is_empty() { "." } else { dir },
            tokens.to_formatted_string(&Locale::en),
            size.to_formatted_string(&Locale::en)
        );
    }

    // --- Step 5: Find and display the top N largest files ---
    // Sort files by the length of their content in descending order.
    files.sort_by(|a, b| b.content.len().cmp(&a.content.len()));

    println!("\nLargest files\n");
    for file in files.iter().take(args.top_file_count) {
        let tokens = estimate_tokens(&file.content);
        println!(
            "- {} (~{} tokens, {} chars)",
            file.filename,
            tokens.to_formatted_string(&Locale::en),
            file.content.len().to_formatted_string(&Locale::en)
        );
    }

    println!("\n✅ Copied to clipboard");
    let mut clipboard = Clipboard::new().context("Failed to initialize clipboard.")?;
    clipboard
        .set_text(&format!("{}", formatted_combined_content_for_clipboard))
        .context("Failed to copy content to clipboard.")?;

    Ok(())
}
