use anyhow::{Context, Result};
use arboard::Clipboard;
use num_format::{Locale, ToFormattedString};
use serde::Deserialize;
use std::process::Command;

/// A struct that represents a single file's data from the yek JSON output.
/// We use serde's `derive` macro to automatically handle deserialization.
#[derive(Deserialize, Debug)]
struct YekFile {
    filename: String,
    content: String,
}

/// Approximate token estimation, assuming 4 characters per token.
pub fn estimate_tokens(text: &str) -> usize {
    text.chars().count() / 4
}

fn main() -> Result<()> {
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

    // Combine all file content into a single string.
    let combined_content: String = files.iter().map(|f| f.content.as_str()).collect();

    let file_count = files.len();
    let token_count = estimate_tokens(&combined_content);

    println!("📂 Files: {}", file_count.to_formatted_string(&Locale::en));
    println!(
        "🧮 Estimated tokens: ~{}",
        token_count.to_formatted_string(&Locale::en)
    );

    // --- Step 6: Find and display the top 10 largest files ---
    // Sort files by the length of their content in descending order.
    files.sort_by(|a, b| b.content.len().cmp(&a.content.len()));

    println!("\nLargest files\n");
    for file in files.iter().take(10) {
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
        .set_text(&combined_content)
        .context("Failed to copy content to clipboard.")?;

    Ok(())
}

