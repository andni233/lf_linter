use clap::Parser;
use content_inspector::inspect;
use glob::Pattern;
use rayon::prelude::*;
use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom};
use std::os::unix::prelude::FileExt;
use std::process::ExitCode;
use std::{fs::File, path::PathBuf};
use walkdir::{DirEntry, WalkDir};

/// Checks that all text files in <PATH> ends with a newline
#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Path to operate on
    #[arg(short, long)]
    path: PathBuf,

    /// Add new line at EOF if missing
    #[arg(short, long)]
    fix: bool,

    /// Rust Patterns for paths to exclude
    #[arg(short, long, value_parser, required = false)]
    exclude: Vec<Pattern>,
}

struct LintTarget {
    entry: DirEntry,
}

impl LintTarget {
    fn new(entry: DirEntry) -> Option<LintTarget> {
        if !entry.file_type().is_file() {
            return None;
        }

        match entry.metadata() {
            Err(_) => return None,
            Ok(metadata) => {
                if metadata.len() == 0 {
                    return None;
                }
            }
        }

        Some(LintTarget { entry })
    }

    /// Read a small sample of the file and attempt to determine if its binary
    fn is_binary(&self) -> bool {
        let mut f = File::open(self.entry.path()).unwrap();
        // Populate the buffer with 1's as 0 is NULL and is used as a heuristic
        // to determine if the file is binary
        let buf: &mut [u8] = &mut [1; 1024];

        f.read(buf).unwrap();

        inspect(buf).is_binary()
    }

    /// Read a small sample of the file and attempt to determine if it uses \r\n as line delimiter
    fn is_crlf(&self) -> bool {
        let mut f = File::open(self.entry.path()).unwrap();
        let buf: &mut [u8] = &mut [0; 1024];

        f.read(buf).unwrap();

        buf.windows(2)
            .position(|window| window == b"\r\n")
            .is_some()
    }

    fn ends_with_newline(&self) -> bool {
        let mut f = File::open(self.entry.path()).unwrap();
        let buf: &mut [u8] = &mut [0];
        let len = f.metadata().unwrap().len();

        f.seek(SeekFrom::Start(len - 1)).unwrap();
        f.read(buf).unwrap();

        // Also detects CRLF (b"\r\n")
        buf == b"\n"
    }

    fn add_newline(&self) {
        let f = OpenOptions::new()
            .write(true)
            .open(self.entry.path())
            .unwrap();
        let len = f.metadata().unwrap().len();

        if self.is_crlf() {
            f.write_all_at(b"\r\n", len).unwrap();
        } else {
            f.write_all_at(b"\n", len).unwrap();
        }
    }
}

fn main() -> ExitCode {
    let args = Args::parse();

    let lint_targets_missing_newline: Vec<LintTarget> =
        WalkDir::new(fs::canonicalize(args.path).unwrap())
            .into_iter()
            .filter_entry(|e| {
                (&args.exclude)
                    .into_iter()
                    .all(|pattern| !pattern.matches_path(e.path()))
            })
            .par_bridge()
            .filter_map(|e| e.ok())
            .filter_map(LintTarget::new)
            .filter(|t| !t.is_binary() && !t.ends_with_newline())
            .collect();

    if !args.fix {
        if !lint_targets_missing_newline.is_empty() {
            println!();
            println!("Following files missing newline at EOF");
            println!();
            for lint_target in lint_targets_missing_newline.iter() {
                println!("  {}", lint_target.entry.path().display());
            }
            println!();
            return ExitCode::FAILURE;
        }
    } else {
        lint_targets_missing_newline
            .into_par_iter()
            .for_each(|t| t.add_newline())
    }

    ExitCode::SUCCESS
}
