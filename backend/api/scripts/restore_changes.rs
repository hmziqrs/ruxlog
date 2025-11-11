#!/usr/bin/env cargo --bin restore_changes

use clap::Parser;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser)]
#[command(name = "restore_changes")]
#[command(about = "Restore files from a zip archive")]
struct Args {
    /// Path to the zip archive to restore
    zip_file: PathBuf,

    /// Overwrite existing files without prompting
    #[arg(short, long)]
    force: bool,

    /// Show what would be restored without doing it
    #[arg(short, long)]
    dry_run: bool,

    /// Target directory (defaults to project root)
    #[arg(long)]
    target_dir: Option<PathBuf>,
}

fn validate_zip_file(zip_path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let full_path = if zip_path.is_absolute() {
        zip_path.to_path_buf()
    } else {
        env::current_dir()?.join(zip_path)
    };

    if !full_path.exists() {
        return Err(format!("Zip file not found: {}", full_path.display()).into());
    }

    Ok(full_path)
}

fn list_zip_contents(zip_path: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = Command::new("unzip")
        .arg("-l")
        .arg(zip_path)
        .output()?;

    if !output.status.success() {
        return Err("Failed to list zip contents".into());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = output_str.lines().collect();
    let mut files = Vec::new();

    let mut in_file_list = false;
    for line in lines {
        if line.contains("Archive:") || line.contains("Length") || line.contains("---") {
            if line.contains("---") {
                in_file_list = true;
            }
            continue;
        }

        if line.trim().matches(|c: char| c.is_ascii_digit()).count() > 0 && line.trim().contains("files") {
            break;
        }

        if in_file_list && !line.trim().is_empty() {
            let parts: Vec<&str> = line.trim().split_whitespace().collect();
            if parts.len() >= 4 {
                let filename = parts[3..].join(" ");
                if !filename.is_empty() && !filename.ends_with('/') {
                    files.push(filename);
                }
            }
        }
    }

    Ok(files)
}

fn check_conflicts(files: &[String], target_dir: &Path) -> Vec<String> {
    let mut conflicts = Vec::new();

    for file in files {
        let target_path = target_dir.join(file);
        if target_path.exists() {
            conflicts.push(file.clone());
        }
    }

    conflicts
}

fn extract_zip(zip_path: &Path, target_dir: &Path, force: bool) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("unzip")
        .arg(if force { "-o" } else { "-n" })
        .arg("-q")
        .arg(zip_path)
        .current_dir(target_dir)
        .output()?;

    if !output.status.success() {
        return Err("Failed to extract zip".into());
    }

    println!("✅ Files extracted successfully!");
    Ok(())
}

fn prompt_confirm(message: &str) -> Result<bool, Box<dyn std::error::Error>> {
    println!("\n{}", message);
    println!("Type 'yes' to continue, or anything else to cancel:");

    print!("> ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let input = input.trim().to_lowercase();
    Ok(input == "yes" || input == "y")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("📦 Restore Archived Files\n");

    let zip_path = validate_zip_file(&args.zip_file)?;
    let root_dir = env::current_exe()?.parent().unwrap().parent().unwrap().to_path_buf();
    let target_dir = args.target_dir.unwrap_or(root_dir);

    println!("📂 Archive: {}", zip_path.file_name().unwrap().to_string_lossy());
    println!("📁 Target directory: {}\n", target_dir.display());

    println!("📋 Archive contents:\n");
    let files = list_zip_contents(&zip_path)?;

    if files.is_empty() {
        println!("⚠️  No files found in archive");
        return Ok(());
    }

    for file in &files {
        println!("   {}", file);
    }
    println!("\n📊 Total files: {}", files.len());

    let conflicts = check_conflicts(&files, &target_dir);

    if !conflicts.is_empty() {
        println!("\n⚠️  {} file(s) would be overwritten:\n", conflicts.len());
        conflicts.iter().take(10).for_each(|file| println!("   {}", file));
        if conflicts.len() > 10 {
            println!("   ... and {} more", conflicts.len() - 10);
        }
    }

    if args.dry_run {
        println!("\n🔍 Dry run mode - no files will be modified");
        if !conflicts.is_empty() {
            println!("\n💡 Use --force to overwrite existing files");
        }
        return Ok(());
    }

    if !conflicts.is_empty() && !args.force {
        println!("\n⚠️  Existing files will NOT be overwritten");
        println!("💡 Use --force to overwrite existing files\n");

        if !prompt_confirm("Continue with restore?")? {
            println!("\n❌ Restore cancelled");
            return Ok(());
        }
    } else if args.force && !conflicts.is_empty() {
        println!("\n⚠️  Force mode: existing files WILL be overwritten\n");

        if !prompt_confirm("⚠️  Are you sure you want to overwrite existing files?")? {
            println!("\n❌ Restore cancelled");
            return Ok(());
        }
    }

    println!("\n📦 Extracting files...\n");
    extract_zip(&zip_path, &target_dir, args.force)?;

    println!("\n✨ Done! {} file(s) restored to {}\n", files.len(), target_dir.display());

    if !args.force && !conflicts.is_empty() {
        println!("💡 Note: {} existing file(s) were skipped", conflicts.len());
        println!("   Use --force to overwrite them\n");
    }

    Ok(())
}