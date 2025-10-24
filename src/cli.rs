use clap::Parser;
use std::path::PathBuf;

/// Command line arguments for the yek-wrapper tool.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Number of top files to display
    #[arg(long, default_value_t = 9)]
    pub top_file_count: usize,

    /// Number of top directories to display
    #[arg(long, default_value_t = 6)]
    pub top_dir_count: usize,

    /// Warn about large files by line count (highlight in orange)
    #[arg(long, default_value_t = 300)]
    pub warn_large_files_by_line_count: usize,

    /// Read path from clipboard and use it as the target directory for yek.
    #[arg(long)]
    pub from_clipboard: bool,

    /// Optional path to run `yek` in. If provided, runs `yek --json .` with this as the working directory.
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,
}
