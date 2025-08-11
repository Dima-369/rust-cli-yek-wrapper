use anyhow::{Context, Result};
use arboard::Clipboard;
use clap::Parser;
use colored::*;
use num_format::{Locale, ToFormattedString};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

mod cli;
use crate::cli::Args;

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
    let args = Args::parse();

    // --- Step 1: Execute `yek --json` (or `yek --json .` in provided PATH) and capture its output ---
    let mut cmd = Command::new("yek");
    cmd.arg("--json");
    if let Some(ref path) = args.path {
        cmd.current_dir(path).arg(".");
    }
    let output = cmd
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

    // Calculate lines for each file and the total raw combined content
    let mut raw_combined_content = String::new();
    let mut total_lines = 0;
    for file in &files {
        let lines_in_file = file.content.lines().count();
        total_lines += lines_in_file;
        raw_combined_content.push_str(&file.content);
    }

    let file_count = files.len();
    let token_count = estimate_tokens(&raw_combined_content);

    println!(
        "~{} tokens / {} files / {} lines",
        token_count.to_formatted_string(&Locale::en),
        file_count.to_formatted_string(&Locale::en),
        total_lines.to_formatted_string(&Locale::en)
    );

    // Prepare formatted combined content for clipboard
    let mut formatted_combined_content_for_clipboard = String::new();
    for (i, file) in files.iter().enumerate() {
        formatted_combined_content_for_clipboard
            .push_str(&format!(">>>> {}\n{}", file.filename, &file.content));
        if i < files.len() - 1 {
            formatted_combined_content_for_clipboard.push('\n');
        }
    }

    // --- Step 4: Aggregate and display top N largest directories ---
    let mut dir_sizes: HashMap<String, (usize, usize)> = HashMap::new(); // Stores (chars, lines)
    for file in &files {
        if let Some(parent) = Path::new(&file.filename).parent() {
            let dir_name = parent.to_string_lossy().into_owned();
            let entry = dir_sizes.entry(dir_name).or_insert((0, 0));
            entry.0 += file.content.len();
            entry.1 += file.content.lines().count(); // Calculate lines directly
        }
    }

    let mut sorted_dirs: Vec<(String, (usize, usize))> = dir_sizes.into_iter().collect();
    sorted_dirs.sort_by(|a, b| b.1.0.cmp(&a.1.0));

    println!(
        "
Largest directories"
    );
    for (dir, (size, lines)) in sorted_dirs.iter().take(args.top_dir_count) {
        if *size == 0 {
            println!("- {} (empty)", if dir.is_empty() { "." } else { dir });
        } else {
            let tokens = estimate_tokens(&String::from_utf8_lossy(&vec![0; *size])); // Approximate tokens for directory size
            println!(
                "- {} (~{} tokens, {} lines, {} chars)",
                if dir.is_empty() { "." } else { dir },
                tokens.to_formatted_string(&Locale::en),
                lines.to_formatted_string(&Locale::en),
                size.to_formatted_string(&Locale::en)
            );
        }
    }

    // --- Step 5: Find and display the top N largest files ---
    // Sort files by the length of their content in descending order.
    files.sort_by(|a, b| b.content.len().cmp(&a.content.len()));

    println!(
        "
Largest files"
    );
    for file in files.iter().take(args.top_file_count) {
        if file.content.is_empty() {
            println!("- {} (empty)", file.filename);
        } else {
            let tokens = estimate_tokens(&file.content);
            let line_count = file.content.lines().count();
            let file_info = format!(
                "- {} (~{} tokens, {} lines, {} chars)",
                file.filename,
                tokens.to_formatted_string(&Locale::en),
                line_count.to_formatted_string(&Locale::en),
                file.content.len().to_formatted_string(&Locale::en)
            );

            if line_count >= args.warn_large_files_by_line_count {
                println!("{}", file_info.bright_yellow());
            } else {
                println!("{file_info}");
            }
        }
    }

    println!(
        "
✅ Copied to clipboard"
    );
    let mut clipboard = Clipboard::new().context("Failed to initialize clipboard.")?;
    clipboard
        .set_text(formatted_combined_content_for_clipboard.to_string())
        .context("Failed to copy content to clipboard.")?;

    Ok(())
}
