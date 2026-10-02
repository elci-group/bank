mod curly_expand;
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

fn __curly_original_main() -> Result<()> {
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

fn main() {
    let raw_args: Vec<String> = std::env::args().collect();
    let mut positions: Vec<usize> = Vec::new();
    let mut fields: Vec<Vec<String>> = Vec::new();
    for (__i, __a) in raw_args.iter().enumerate() {
        if __a == "--mode" {
            if let Some(__v) = raw_args.get(__i + 1) {
                positions.push(__i + 1);
                fields.push(curly_expand::expand_or_literal(__v));
            }
            break;
        } else if let Some(__v) = __a.strip_prefix("--mode=") {
            positions.push(__i);
            fields.push(
                curly_expand::expand_or_literal(__v)
                    .into_iter()
                    .map(|v| format!("--mode={}", v))
                    .collect(),
            );
            break;
        }
    }
    for (__i, __a) in raw_args.iter().enumerate() {
        if __a == "--date" {
            if let Some(__v) = raw_args.get(__i + 1) {
                positions.push(__i + 1);
                fields.push(curly_expand::expand_or_literal(__v));
            }
            break;
        } else if let Some(__v) = __a.strip_prefix("--date=") {
            positions.push(__i);
            fields.push(
                curly_expand::expand_or_literal(__v)
                    .into_iter()
                    .map(|v| format!("--date={}", v))
                    .collect(),
            );
            break;
        }
    }
    for (__i, __a) in raw_args.iter().enumerate() {
        if __a == "--timestamp" {
            if let Some(__v) = raw_args.get(__i + 1) {
                positions.push(__i + 1);
                fields.push(curly_expand::expand_or_literal(__v));
            }
            break;
        } else if let Some(__v) = __a.strip_prefix("--timestamp=") {
            positions.push(__i);
            fields.push(
                curly_expand::expand_or_literal(__v)
                    .into_iter()
                    .map(|v| format!("--timestamp={}", v))
                    .collect(),
            );
            break;
        }
    }
    for (__i, __a) in raw_args.iter().enumerate() {
        if __a == "--reference" {
            if let Some(__v) = raw_args.get(__i + 1) {
                positions.push(__i + 1);
                fields.push(curly_expand::expand_or_literal(__v));
            }
            break;
        } else if let Some(__v) = __a.strip_prefix("--reference=") {
            positions.push(__i);
            fields.push(
                curly_expand::expand_or_literal(__v)
                    .into_iter()
                    .map(|v| format!("--reference={}", v))
                    .collect(),
            );
            break;
        }
    }
    if let Some(__v) = raw_args.get(1) {
        if !__v.starts_with('-') {
            positions.push(1);
            fields.push(curly_expand::expand_or_literal(__v));
        }
    }

    if fields.is_empty() || fields.iter().all(|f| f.len() <= 1) {
        if let Err(e) = __curly_original_main() {
            eprintln!("{:#}", e);
            std::process::exit(1);
        }
        return;
    }

    let combos = curly_expand::cartesian(&fields);
    let exe = std::env::current_exe().expect("resolve current exe");
    let mut had_failure = false;
    for combo in &combos {
        let mut new_args = raw_args.clone();
        for (slot, value) in positions.iter().zip(combo.iter()) {
            new_args[*slot] = value.clone();
        }
        let status = std::process::Command::new(&exe)
            .args(&new_args[1..])
            .status()
            .expect("failed to re-exec self");
        if !status.success() {
            had_failure = true;
        }
    }
    if had_failure {
        std::process::exit(1);
    }
}
