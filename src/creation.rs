use crate::args::Args;
use crate::output::colored_path;
use anyhow::{Context, Result};
use form3::ansi::Color;
use dialoguer::{theme::ColorfulTheme, Select};
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub enum CreationType {
    File,
    Directory,
}

pub fn determine_creation_type(args: &Args, path: &Path, path_str: &str) -> Result<CreationType> {
    // Explicit flags take precedence
    if args.directory {
        return Ok(CreationType::Directory);
    }

    if args.file {
        return Ok(CreationType::File);
    }

    // Check if path already exists
    if path.exists() {
        if path.is_dir() {
            return Ok(CreationType::Directory);
        } else {
            return Ok(CreationType::File);
        }
    }

    // Heuristics for ambiguous paths
    if let Some(extension) = path.extension() {
        if !extension.is_empty() {
            return Ok(CreationType::File);
        }
    }

    // Path ends with separator -> directory
    if path_str.ends_with('/') || path_str.ends_with('\\') {
        return Ok(CreationType::Directory);
    }

    // Interactive mode or auto-detection
    if args.interactive {
        let choices = vec!["File", "Directory"];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("What should '{}' be?", path.display()))
            .items(&choices)
            .default(0)
            .interact()?;

        match selection {
            0 => Ok(CreationType::File),
            1 => Ok(CreationType::Directory),
            _ => unreachable!(),
        }
    } else {
        // Default to file for ambiguous cases
        Ok(CreationType::File)
    }
}

pub fn create_file(path: &Path, args: &Args) -> Result<()> {
    if path.exists() {
        if args.verbose {
            println!("File already exists: {}", colored_path(path, Color::Yellow));
        }
        // Don't update timestamps here - will be handled by set_file_times if needed
    } else {
        fs::File::create(path)
            .with_context(|| format!("Failed to create file {}", path.display()))?;
    }
    Ok(())
}

pub fn create_directory(path: &Path, args: &Args) -> Result<()> {
    if path.exists() {
        if path.is_dir() {
            if args.verbose {
                println!("Directory already exists: {}", colored_path(path, Color::Yellow));
            }
        } else {
            anyhow::bail!("Path exists but is not a directory: {}", path.display());
        }
    } else {
        fs::create_dir(path)
            .with_context(|| format!("Failed to create directory {}", path.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::create_test_args;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_create_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let mut args = create_test_args(vec![file_path.to_str().unwrap().to_string()]);
        args.file = true;

        create_file(&file_path, &args).unwrap();
        assert!(file_path.exists());
        assert!(file_path.is_file());
    }

    #[test]
    fn test_create_directory() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("test_dir");

        let mut args = create_test_args(vec![dir_path.to_str().unwrap().to_string()]);
        args.directory = true;

        create_directory(&dir_path, &args).unwrap();
        assert!(dir_path.exists());
        assert!(dir_path.is_dir());
    }

    #[test]
    fn test_determine_creation_type_with_extension() {
        let args = create_test_args(vec!["test.txt".to_string()]);
        let path = PathBuf::from("test.txt");
        let creation_type = determine_creation_type(&args, &path, "test.txt").unwrap();

        match creation_type {
            CreationType::File => (),
            _ => panic!("Should be file"),
        }
    }

    #[test]
    fn test_determine_creation_type_with_trailing_slash() {
        let args = create_test_args(vec!["test_dir/".to_string()]);
        let path = PathBuf::from("test_dir");
        let creation_type = determine_creation_type(&args, &path, "test_dir/").unwrap();

        match creation_type {
            CreationType::Directory => (),
            _ => panic!("Should be directory"),
        }
    }
}
