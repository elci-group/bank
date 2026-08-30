mod args;
mod creation;
mod output;
mod permissions;
#[cfg(test)]
mod test_helpers;
mod timestamps;

use anyhow::{Context, Result};
use args::{validate_arguments, Args};
use clap::Parser;
use creation::{create_directory, create_file, determine_creation_type, CreationType};
use form3::ansi::Color;
use form3::compat::Colorize;
use output::{colored_path, report_progress};
use permissions::set_permissions;
use std::fs;
use std::path::PathBuf;
use timestamps::{get_time_spec, parse_timestamp, set_file_times};

fn main() -> Result<()> {
    let args = Args::parse();

    validate_arguments(&args)?;

    if args.verbose {
        println!("{} {}", "Bank".bright_green().bold(), "v0.2.0".cyan());
        if args.paths.len() > 1 {
            println!("Processing {} paths...", args.paths.len().to_string().cyan());
        }
    }

    for path_str in &args.paths {
        process_single_path(path_str, &args)?;
    }

    Ok(())
}

fn process_single_path(path_str: &str, args: &Args) -> Result<()> {
    let path = PathBuf::from(path_str);

    // Parse custom timestamp if provided
    let custom_time = parse_timestamp(args)?;

    // Check no-create mode
    if args.no_create {
        if !path.exists() {
            if args.verbose {
                println!("Skipping non-existent path in no-create mode: {}", colored_path(&path, Color::Yellow));
            }
            return Ok(());
        }

        // Only update timestamps for existing files/directories
        let time_spec = get_time_spec(args, custom_time)?;
        set_file_times(&path, &time_spec, args)?;

        report_progress(&path, args, "Updated timestamps:");
        return Ok(());
    }

    // Determine what to create
    let creation_type = determine_creation_type(args, &path, path_str)?;

    if args.verbose {
        match creation_type {
            CreationType::File => println!("Creating file: {}", colored_path(&path, Color::Yellow)),
            CreationType::Directory => println!("Creating directory: {}", colored_path(&path, Color::Yellow)),
        }
    }

    // Create parents if needed
    if args.parents {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create parent directories for {}", path.display()))?;
                if args.verbose {
                    println!("Created parent directories: {}", colored_path(parent, Color::Green));
                }
            }
        }
    }

    // Create the target
    match creation_type {
        CreationType::File => create_file(&path, args)?,
        CreationType::Directory => create_directory(&path, args)?,
    }

    // Set custom timestamps if specified
    if custom_time.is_some() || args.access_time_only || args.modification_time_only {
        let time_spec = get_time_spec(args, custom_time)?;
        set_file_times(&path, &time_spec, args)?;
    }

    // Set permissions if specified
    if let Some(mode_str) = &args.mode {
        set_permissions(&path, mode_str, args.verbose)?;
    }

    report_progress(&path, args, "Created:");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::create_test_args;
    use tempfile::TempDir;

    #[test]
    fn test_multiple_files() {
        let temp_dir = TempDir::new().unwrap();
        let file1_path = temp_dir.path().join("file1.txt");
        let file2_path = temp_dir.path().join("file2.txt");

        let mut args = create_test_args(vec![
            file1_path.to_str().unwrap().to_string(),
            file2_path.to_str().unwrap().to_string(),
        ]);
        args.file = true;

        process_single_path(&args.paths[0], &args).unwrap();
        process_single_path(&args.paths[1], &args).unwrap();

        assert!(file1_path.exists());
        assert!(file1_path.is_file());
        assert!(file2_path.exists());
        assert!(file2_path.is_file());
    }

    #[test]
    fn test_no_create_mode() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("existing.txt");
        let nonexistent_path = temp_dir.path().join("nonexistent.txt");

        // Create the file first
        std::fs::File::create(&file_path).unwrap();

        let mut args = create_test_args(vec![file_path.to_str().unwrap().to_string()]);
        args.no_create = true;

        // Should succeed for existing file
        process_single_path(file_path.to_str().unwrap(), &args).unwrap();

        // Should not create nonexistent file
        let mut args2 = create_test_args(vec![nonexistent_path.to_str().unwrap().to_string()]);
        args2.no_create = true;
        process_single_path(nonexistent_path.to_str().unwrap(), &args2).unwrap();

        assert!(!nonexistent_path.exists());
    }
}
