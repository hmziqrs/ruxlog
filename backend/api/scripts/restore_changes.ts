#!/usr/bin/env bun
/**
 * Restore Archived Files Script
 * 
 * This script:
 * 1. Takes a zip file path as argument
 * 2. Lists the contents for review
 * 3. Optionally extracts files to restore changes
 * 4. Preserves directory structure
 */

import { $ } from "bun";
import { existsSync } from "fs";
import { join, basename } from "path";

const ROOT_DIR = join(import.meta.dir, "..");

interface RestoreOptions {
  zipPath: string;
  force?: boolean;
  dryRun?: boolean;
  targetDir?: string;
}

/**
 * Validate that the zip file exists
 */
function validateZipFile(zipPath: string): string {
  let fullPath = zipPath;

  // If relative path, make it absolute
  if (!zipPath.startsWith("/")) {
    fullPath = join(process.cwd(), zipPath);
  }

  if (!existsSync(fullPath)) {
    throw new Error(`Zip file not found: ${fullPath}`);
  }

  return fullPath;
}

/**
 * List contents of the zip file
 */
async function listZipContents(zipPath: string): Promise<string[]> {
  try {
    const output = await $`unzip -l ${zipPath}`.text();
    
    // Parse unzip -l output to get file list
    const lines = output.split("\n");
    const files: string[] = [];
    
    let inFileList = false;
    for (const line of lines) {
      // Skip header lines
      if (line.includes("Archive:") || line.includes("Length") || line.includes("---")) {
        if (line.includes("---")) {
          inFileList = true;
        }
        continue;
      }
      
      // Stop at footer
      if (line.trim().match(/^\d+\s+\d+\s+files?$/)) {
        break;
      }
      
      if (inFileList && line.trim()) {
        // Extract filename (last column after date/time)
        const parts = line.trim().split(/\s+/);
        if (parts.length >= 4) {
          const filename = parts.slice(3).join(" ");
          if (filename && !filename.endsWith("/")) {
            files.push(filename);
          }
        }
      }
    }
    
    return files;
  } catch (error) {
    console.error("Error listing zip contents:", error);
    throw error;
  }
}

/**
 * Check which files would be overwritten
 */
function checkConflicts(files: string[], targetDir: string): string[] {
  const conflicts: string[] = [];
  
  for (const file of files) {
    const targetPath = join(targetDir, file);
    if (existsSync(targetPath)) {
      conflicts.push(file);
    }
  }
  
  return conflicts;
}

/**
 * Extract the zip file
 */
async function extractZip(zipPath: string, targetDir: string, force: boolean): Promise<void> {
  try {
    process.chdir(targetDir);
    
    if (force) {
      // Overwrite existing files
      await $`unzip -o -q ${zipPath}`;
    } else {
      // Don't overwrite existing files
      await $`unzip -n -q ${zipPath}`;
    }
    
    console.log("✅ Files extracted successfully!");
  } catch (error) {
    console.error("Error extracting zip:", error);
    throw error;
  }
}

/**
 * Display help message
 */
function displayHelp() {
  console.log(`
📦 Restore Archived Files

Usage:
  bun scripts/restore_changes.ts <zip-file> [options]

Arguments:
  <zip-file>          Path to the zip archive to restore

Options:
  --force, -f         Overwrite existing files without prompting
  --dry-run, -d       Show what would be restored without doing it
  --target-dir <dir>  Target directory (defaults to project root)
  --help, -h          Show this help message

Examples:
  # List contents of an archive (dry run)
  bun scripts/restore_changes.ts backups/changes/changes-2025-11-01-14-30-abc123.zip --dry-run

  # Restore files (will skip existing files)
  bun scripts/restore_changes.ts backups/changes/changes-2025-11-01-14-30-abc123.zip

  # Restore and overwrite existing files
  bun scripts/restore_changes.ts backups/changes/changes-2025-11-01-14-30-abc123.zip --force

  # Restore to a specific directory
  bun scripts/restore_changes.ts archive.zip --target-dir /path/to/restore
`);
}

/**
 * Parse command line arguments
 */
function parseArgs(): RestoreOptions | null {
  const args = process.argv.slice(2);
  
  if (args.length === 0 || args.includes("--help") || args.includes("-h")) {
    displayHelp();
    return null;
  }
  
  const options: RestoreOptions = {
    zipPath: args[0],
    force: args.includes("--force") || args.includes("-f"),
    dryRun: args.includes("--dry-run") || args.includes("-d"),
  };
  
  // Check for target directory
  const targetDirIndex = args.indexOf("--target-dir");
  if (targetDirIndex !== -1 && args[targetDirIndex + 1]) {
    options.targetDir = args[targetDirIndex + 1];
  }
  
  return options;
}

/**
 * Prompt user for confirmation
 */
async function promptConfirm(message: string): Promise<boolean> {
  console.log(`\n${message}`);
  console.log("Type 'yes' to continue, or anything else to cancel:");
  
  // Read from stdin
  const input = await new Promise<string>((resolve) => {
    process.stdin.once("data", (data) => {
      resolve(data.toString().trim().toLowerCase());
    });
  });
  
  return input === "yes" || input === "y";
}

/**
 * Main execution
 */
async function main() {
  console.log("📦 Restore Archived Files\n");
  
  try {
    const options = parseArgs();
    if (!options) {
      process.exit(0);
    }
    
    // Validate zip file
    const zipPath = validateZipFile(options.zipPath);
    const targetDir = options.targetDir || ROOT_DIR;
    
    console.log(`📂 Archive: ${basename(zipPath)}`);
    console.log(`📁 Target directory: ${targetDir}\n`);
    
    // List contents
    console.log("📋 Archive contents:\n");
    const files = await listZipContents(zipPath);
    
    if (files.length === 0) {
      console.log("⚠️  No files found in archive");
      process.exit(0);
    }
    
    files.forEach((file) => console.log(`   ${file}`));
    console.log(`\n📊 Total files: ${files.length}`);
    
    // Check for conflicts
    const conflicts = checkConflicts(files, targetDir);
    
    if (conflicts.length > 0) {
      console.log(`\n⚠️  ${conflicts.length} file(s) would be overwritten:\n`);
      conflicts.slice(0, 10).forEach((file) => console.log(`   ${file}`));
      if (conflicts.length > 10) {
        console.log(`   ... and ${conflicts.length - 10} more`);
      }
    }
    
    // Dry run - just show what would happen
    if (options.dryRun) {
      console.log("\n🔍 Dry run mode - no files will be modified");
      if (conflicts.length > 0) {
        console.log(`\n💡 Use --force to overwrite existing files`);
      }
      process.exit(0);
    }
    
    // Confirm extraction
    if (conflicts.length > 0 && !options.force) {
      console.log("\n⚠️  Existing files will NOT be overwritten");
      console.log("💡 Use --force to overwrite existing files\n");
      
      const confirmed = await promptConfirm("Continue with restore?");
      if (!confirmed) {
        console.log("\n❌ Restore cancelled");
        process.exit(0);
      }
    } else if (options.force && conflicts.length > 0) {
      console.log("\n⚠️  Force mode: existing files WILL be overwritten\n");
      
      const confirmed = await promptConfirm("⚠️  Are you sure you want to overwrite existing files?");
      if (!confirmed) {
        console.log("\n❌ Restore cancelled");
        process.exit(0);
      }
    }
    
    // Extract files
    console.log("\n📦 Extracting files...\n");
    await extractZip(zipPath, targetDir, options.force || false);
    
    console.log(`\n✨ Done! ${files.length} file(s) restored to ${targetDir}\n`);
    
    if (!options.force && conflicts.length > 0) {
      console.log(`💡 Note: ${conflicts.length} existing file(s) were skipped`);
      console.log(`   Use --force to overwrite them\n`);
    }
    
  } catch (error) {
    console.error("\n❌ Error:", error);
    process.exit(1);
  }
}

// Run if executed directly
if (import.meta.main) {
  main();
}

export { validateZipFile, listZipContents, extractZip };
