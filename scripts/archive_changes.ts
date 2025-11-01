#!/usr/bin/env bun
/**
 * Archive Changed Files Script
 * 
 * This script:
 * 1. Detects git-modified files (staged and unstaged)
 * 2. Creates a zip archive with a hash-based filename
 * 3. Saves it to the backups/ directory
 */

import { $ } from "bun";
import { createHash } from "crypto";
// @ts-ignore: Running under Bun/Node runtime - fs types may not be available in this workspace
import { existsSync, mkdirSync, statSync } from "fs";
import { join } from "path";
import { file } from "bun";

const BACKUPS_DIR = join(import.meta.dir, "..", "backups", "changes");
const ROOT_DIR = join(import.meta.dir, "..");

interface GitStatus {
  staged: string[];
  unstaged: string[];
  untracked: string[];
}

/**
 * Get git status and categorize files
 */
async function getGitStatus(): Promise<GitStatus> {
  try {
    // Get staged files
    const stagedOutput = await $`git diff --cached --name-only --diff-filter=ACMR`.text();
    const staged = stagedOutput
      .trim()
      .split("\n")
      .filter((f) => f.length > 0);

    // Get unstaged modified files
    const unstagedOutput = await $`git diff --name-only --diff-filter=ACMR`.text();
    const unstaged = unstagedOutput
      .trim()
      .split("\n")
      .filter((f) => f.length > 0);

    // Get untracked files
    const untrackedOutput = await $`git ls-files --others --exclude-standard`.text();
    const untracked = untrackedOutput
      .trim()
      .split("\n")
      .filter((f) => f.length > 0);

    return { staged, unstaged, untracked };
  } catch (error) {
    console.error("Error getting git status:", error);
    throw error;
  }
}

/**
 * Generate a hash from the file list and current timestamp
 */
function generateHash(files: string[]): string {
  const timestamp = Date.now();
  const content = files.join("\n") + timestamp;
  return createHash("sha256").update(content).digest("hex").substring(0, 12);
}

/**
 * Create a zip archive of the specified files
 */
async function createArchive(files: string[], hash: string): Promise<string> {
  if (files.length === 0) {
    throw new Error("No files to archive");
  }

  // Ensure backups directory exists
  if (!existsSync(BACKUPS_DIR)) {
    mkdirSync(BACKUPS_DIR, { recursive: true });
  }

  const timestamp = new Date().toISOString().replace(/[:.]/g, "-").split("T")[0];
  const timeHM = new Date().toTimeString().split(" ")[0].replace(/:/g, "-");
  const zipFilename = `changes-${timestamp}-${timeHM}-${hash}.zip`;
  const zipPath = join(BACKUPS_DIR, zipFilename);

  console.log(`\n📦 Creating archive: ${zipFilename}`);
  console.log(`📂 Location: ${BACKUPS_DIR}\n`);

  try {
    // Change to root directory to preserve directory structure in zip
    process.chdir(ROOT_DIR);

    // Create a temporary file list for zip to handle spaces in filenames
    const fileListPath = join(BACKUPS_DIR, `.filelist-${hash}.tmp`);
    await Bun.write(fileListPath, files.join("\n"));

    // Create zip with directory structure preserved (-r for recursive, reading from file list)
    // Using -@ to read file list from stdin to handle special characters in filenames
    await $`cat ${fileListPath} | zip -q -@ ${zipPath}`.quiet();

    // Clean up temp file
    await $`rm ${fileListPath}`.quiet();

  // Get archive size
  const stats = statSync(zipPath);
  const sizeKB = (stats.size / 1024).toFixed(2);

    console.log(`✅ Archive created successfully!`);
    console.log(`📊 Size: ${sizeKB} KB`);
    console.log(`📁 Files archived: ${files.length}`);
    console.log(`📂 Directory structure preserved`);

    return zipPath;
  } catch (error) {
    console.error("Error creating archive:", error);
    throw error;
  }
}

/**
 * Display file categorization
 */
function displayFiles(status: GitStatus) {
  const allFiles = [...new Set([...status.staged, ...status.unstaged, ...status.untracked])];

  if (allFiles.length === 0) {
    console.log("✨ No changed files detected");
    return;
  }

  console.log("\n📝 Changed Files:\n");

  if (status.staged.length > 0) {
    console.log("  Staged:");
    status.staged.forEach((f) => console.log(`    ✓ ${f}`));
  }

  if (status.unstaged.length > 0) {
    console.log("\n  Modified (unstaged):");
    status.unstaged.forEach((f) => console.log(`    • ${f}`));
  }

  if (status.untracked.length > 0) {
    console.log("\n  Untracked:");
    status.untracked.forEach((f) => console.log(`    ? ${f}`));
  }

  console.log(`\n📊 Total files: ${allFiles.length}`);
}

/**
 * Main execution
 */
async function main() {
  console.log("🔍 Checking for changed files...\n");

  try {
    // Get git status
    const status = await getGitStatus();
    const allFiles = [...new Set([...status.staged, ...status.unstaged, ...status.untracked])];

    // Display files
    displayFiles(status);

    if (allFiles.length === 0) {
      console.log("\n💡 No files to archive. Make some changes first!");
      process.exit(0);
    }

    // Filter out files that don't exist or are too large
    const validFiles: string[] = [];
    for (const filePath of allFiles) {
      const fullPath = join(ROOT_DIR, filePath);
      if (existsSync(fullPath)) {
        const stats = statSync(fullPath);
        // Skip files larger than 50MB
        if (stats.size < 50 * 1024 * 1024) {
          validFiles.push(filePath);
        } else {
          console.log(`⚠️  Skipping large file: ${filePath} (${(stats.size / 1024 / 1024).toFixed(2)} MB)`);
        }
      }
    }

    if (validFiles.length === 0) {
      console.log("\n⚠️  No valid files to archive");
      process.exit(0);
    }

    // Generate hash and create archive
    const hash = generateHash(validFiles);
    const zipPath = await createArchive(validFiles, hash);

    console.log(`\n✨ Done!\n`);
    console.log(`📦 Archive: ${zipPath}`);
    console.log(`\n💡 To extract:`);
    console.log(`   unzip "${zipPath}"\n`);
  } catch (error) {
    console.error("\n❌ Error:", error);
    process.exit(1);
  }
}

// Run if executed directly
if (import.meta.main) {
  main();
}

export { getGitStatus, createArchive, generateHash };
