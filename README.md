A wrapper around `yek` for easier stats.

# Example output

```
📂 Files: 4
🧮 Estimated tokens: ~1,472

Largest directories

- src (~1,174 tokens, 4,696 chars)
- . (~301 tokens, 1,204 chars)

Largest files

- src/main.rs (~1,171 tokens, 4,696 chars)
- TODO.md (~227 tokens, 910 chars)
- Cargo.toml (~63 tokens, 253 chars)
- README.md (~10 tokens, 41 chars)

✅ Copied to clipboard
```

# yek-wrapper Help

```
Command line arguments for the yek-wrapper tool

Usage: yek-wrapper [OPTIONS]

Options:
      --top-file-count <TOP_FILE_COUNT>
          Number of top files to display [default: 9]
      --top-dir-count <TOP_DIR_COUNT>
          Number of top directories to display [default: 6]
      --warn-large-files-by-line-count <WARN_LARGE_FILES_BY_LINE_COUNT>
          Warn about large files by line count (highlight in orange) [default: 300]
  -h, --help
          Print help
  -V, --version
          Print version
```