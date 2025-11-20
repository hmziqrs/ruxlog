#!/usr/bin/env cargo --bin archive_changes

use clap::Parser;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024; // 50MB

#[derive(Parser)]
#[command(name = "archive_changes")]
#[command(about = "Archive git-modified files to a zip archive")]
struct Args {
    /// Output directory for archives (default: ../backups)
    #[arg(short, long)]
    output: Option<PathBuf>,
}

#[derive(Debug)]
struct GitStatus {
    staged: Vec<String>,
    unstaged: Vec<String>,
    untracked: Vec<String>,
}

fn get_git_status() -> Result<GitStatus, Box<dyn std::error::Error>> {
    let staged_output = Command::new("git")
        .args(&["diff", "--cached", "--name-only", "--diff-filter=ACMR"])
        .output()?;

    let staged = String::from_utf8_lossy(&staged_output.stdout)
        .lines()
        .filter(|s| !s.trim().is_empty())
        .map(String::from)
        .collect();

    let unstaged_output = Command::new("git")
        .args(&["diff", "--name-only", "--diff-filter=ACMR"])
        .output()?;

    let unstaged = String::from_utf8_lossy(&unstaged_output.stdout)
        .lines()
        .filter(|s| !s.trim().is_empty())
        .map(String::from)
        .collect();

    let untracked_output = Command::new("git")
        .args(&["ls-files", "--others", "--exclude-standard"])
        .output()?;

    let untracked = String::from_utf8_lossy(&untracked_output.stdout)
        .lines()
        .filter(|s| !s.trim().is_empty())
        .map(String::from)
        .collect();

    Ok(GitStatus {
        staged,
        unstaged,
        untracked,
    })
}

fn generate_hash(files: &[String]) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let mut hasher = Sha256::new();
    for file in files {
        hasher.update(file.as_bytes());
        hasher.update(b"\n");
    }
    hasher.update(timestamp.to_string().as_bytes());

    hex::encode(hasher.finalize())[..8].to_string()
}

fn create_archive(
    files: Vec<String>,
    hash: &str,
    backups_dir: &Path,
    root_dir: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if files.is_empty() {
        return Err("No files to archive".into());
    }

    if !backups_dir.exists() {
        fs::create_dir_all(backups_dir)?;
    }

    let zip_filename = format!("{}.zip", hash);
    let zip_path = backups_dir.join(&zip_filename);

    println!("\n📦 Creating archive: {}", zip_filename);
    println!("📂 Location: {}\n", backups_dir.display());

    let temp_file_list = backups_dir.join(format!(".filelist-{}.tmp", hash));
    fs::write(&temp_file_list, files.join("\n"))?;

    let status = Command::new("sh")
        .current_dir(root_dir)
        .arg("-c")
        .arg(&format!(
            "cat {} | zip -q -@ {}",
            temp_file_list.display(),
            zip_path.display()
        ))
        .status()?;

    fs::remove_file(&temp_file_list)?;

    if !status.success() {
        return Err("Failed to create zip archive".into());
    }

    let metadata = fs::metadata(&zip_path)?;
    let size_kb = metadata.len() as f64 / 1024.0;

    println!("✅ Archive created successfully!");
    println!("📊 Size: {:.2} KB", size_kb);
    println!("📁 Files archived: {}", files.len());
    println!("📂 Directory structure preserved");

    Ok(zip_path)
}

fn display_files(status: &GitStatus) {
    let all_files: Vec<String> = status
        .staged
        .iter()
        .chain(&status.unstaged)
        .chain(&status.untracked)
        .cloned()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    if all_files.is_empty() {
        println!("✨ No changed files detected");
        return;
    }

    println!("\n📝 Changed Files:\n");

    if !status.staged.is_empty() {
        println!("  Staged:");
        for f in &status.staged {
            println!("    ✓ {}", f);
        }
    }

    if !status.unstaged.is_empty() {
        println!("\n  Modified (unstaged):");
        for f in &status.unstaged {
            println!("    • {}", f);
        }
    }

    if !status.untracked.is_empty() {
        println!("\n  Untracked:");
        for f in &status.untracked {
            println!("    ? {}", f);
        }
    }

    println!("\n📊 Total files: {}", all_files.len());
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("🔍 Checking for changed files...\n");

    let root_dir = env::current_exe()?
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let backups_dir = args.output.unwrap_or_else(|| root_dir.join("backups"));

    let status = get_git_status()?;
    let all_files: Vec<String> = status
        .staged
        .iter()
        .chain(&status.unstaged)
        .chain(&status.untracked)
        .cloned()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    display_files(&status);

    if all_files.is_empty() {
        println!("\n💡 No files to archive. Make some changes first!");
        return Ok(());
    }

    let mut valid_files = Vec::new();
    for file_path in &all_files {
        let full_path = root_dir.join(file_path);
        if full_path.exists() {
            let metadata = fs::metadata(&full_path)?;
            if metadata.len() < MAX_FILE_SIZE {
                valid_files.push(file_path.clone());
            } else {
                let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
                println!("⚠️  Skipping large file: {} ({:.2} MB)", file_path, size_mb);
            }
        }
    }

    if valid_files.is_empty() {
        println!("\n⚠️  No valid files to archive");
        return Ok(());
    }

    let hash = generate_hash(&valid_files);
    let zip_path = create_archive(valid_files, &hash, &backups_dir, &root_dir)?;

    println!("\n✨ Done!\n");
    println!("📦 Archive: {}", zip_path.display());
    println!("\n💡 To extract:");
    println!("   unzip \"{}\"\n", zip_path.display());

    Ok(())
}
