use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, NaiveDateTime, Utc};
use clap::Parser;
use colored::*;
use dialoguer::{theme::ColorfulTheme, Select};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Bank: A comprehensive command-line utility combining mkdir, touch, and advanced filesystem operations
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The paths to create (files or directories)
    #[arg(value_name = "PATH", required = true)]
    paths: Vec<String>,

    /// Force creation as directory (mkdir mode)
    #[arg(short = 'd', long = "directory")]
    directory: bool,

    /// Force creation as file (touch mode)
    #[arg(short = 'f', long = "file")]
    file: bool,

    /// Create parent directories as needed
    #[arg(short = 'p', long = "parents")]
    parents: bool,

    /// Set file/directory permissions (octal format, e.g., 755)
    #[arg(short = 'm', long = "mode")]
    mode: Option<String>,

    /// Interactive mode for ambiguous paths
    #[arg(short = 'i', long = "interactive")]
    interactive: bool,

    /// Verbose output
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    /// Do not create files, only update timestamps if they exist
    #[arg(short = 'c', long = "no-create")]
    no_create: bool,

    /// Parse date string and use it instead of current time
    #[arg(long = "date", value_name = "STRING")]
    date: Option<String>,

    /// Use timestamp format [[CC]YY]MMDDhhmm[.ss] instead of current time
    #[arg(short = 't', long = "timestamp", value_name = "STAMP")]
    timestamp: Option<String>,

    /// Use this file's times instead of current time
    #[arg(short = 'r', long = "reference", value_name = "FILE")]
    reference: Option<String>,

    /// Change only the access time
    #[arg(short = 'a', long = "atime")]
    access_time_only: bool,

    /// Change only the modification time
    #[arg(long = "mtime")]
    modification_time_only: bool,

    /// Affect symbolic links instead of referenced files
    #[arg(long = "no-dereference")]
    no_dereference: bool,
}

#[derive(Debug)]
enum CreationType {
    File,
    Directory,
}

#[derive(Debug)]
struct TimeSpec {
    access_time: Option<SystemTime>,
    modification_time: Option<SystemTime>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Validate argument combinations
    validate_arguments(&args)?;

    if args.verbose {
        println!("{} {}", "Bank".bright_green().bold(), "v0.2.0".cyan());
        if args.paths.len() > 1 {
            println!("Processing {} paths...", args.paths.len().to_string().cyan());
        }
    }

    // Process each path
    for path_str in &args.paths {
        process_single_path(path_str, &args)?;
    }

    Ok(())
}

/// Validate argument combinations
fn validate_arguments(args: &Args) -> Result<()> {
    // Check for conflicting directory/file flags
    if args.directory && args.file {
        anyhow::bail!("Cannot specify both --directory and --file flags");
    }
    
    // Check for conflicting time specification flags  
    let time_sources = [args.date.is_some(), args.timestamp.is_some(), args.reference.is_some()];
    let time_source_count = time_sources.iter().filter(|&&x| x).count();
    if time_source_count > 1 {
        anyhow::bail!("Cannot specify multiple time sources (--date, --timestamp, --reference)");
    }
    
    // Check for conflicting access/modification time flags
    if args.access_time_only && args.modification_time_only {
        anyhow::bail!("Cannot specify both --atime and --mtime flags");
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
                println!("Skipping non-existent path in no-create mode: {}", path.display().to_string().yellow());
            }
            return Ok(());
        }
        
        // Only update timestamps for existing files/directories
        let time_spec = get_time_spec(args, custom_time)?;
        set_file_times(&path, &time_spec, args)?;
        
        if args.verbose {
            println!("{} Updated timestamps: {}", "✓".bright_green(), path.display().to_string().green());
        } else if args.paths.len() > 1 {
            println!("{} {}", "✓".bright_green(), path.display().to_string().green());
        }
        return Ok(());
    }
    
    // Determine what to create
    let creation_type = determine_creation_type(args, &path, path_str)?;
    
    if args.verbose {
        match creation_type {
            CreationType::File => println!("Creating file: {}", path.display().to_string().yellow()),
            CreationType::Directory => println!("Creating directory: {}", path.display().to_string().yellow()),
        }
    }

    // Create parents if needed
    if args.parents {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create parent directories for {}", path.display()))?;
                if args.verbose {
                    println!("Created parent directories: {}", parent.display().to_string().green());
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

    if args.verbose {
        println!("{} Created: {}", "✓".bright_green(), path.display().to_string().green());
    } else if args.paths.len() > 1 {
        // Show minimal progress for multiple files when not verbose
        println!("{} {}", "✓".bright_green(), path.display().to_string().green());
    }

    Ok(())
}

fn determine_creation_type(args: &Args, path: &Path, path_str: &str) -> Result<CreationType> {
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

fn create_file(path: &Path, args: &Args) -> Result<()> {
    if path.exists() {
        if args.verbose {
            println!("File already exists: {}", path.display().to_string().yellow());
        }
        // Don't update timestamps here - will be handled by set_file_times if needed
    } else {
        fs::File::create(path)
            .with_context(|| format!("Failed to create file {}", path.display()))?;
    }
    Ok(())
}

fn create_directory(path: &Path, args: &Args) -> Result<()> {
    if path.exists() {
        if path.is_dir() {
            if args.verbose {
                println!("Directory already exists: {}", path.display().to_string().yellow());
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

fn set_permissions(path: &Path, mode_str: &str, verbose: bool) -> Result<()> {
    let mode = u32::from_str_radix(mode_str, 8)
        .with_context(|| format!("Invalid mode format: {}", mode_str))?;

    let permissions = fs::Permissions::from_mode(mode);
    fs::set_permissions(path, permissions)
        .with_context(|| format!("Failed to set permissions for {}", path.display()))?;

    if verbose {
        println!("Set permissions to {} for {}", mode_str.green(), path.display());
    }

    Ok(())
}

/// Set file timestamps with symlink handling support
fn set_file_times(path: &Path, time_spec: &TimeSpec, args: &Args) -> Result<()> {
    // Handle symlinks if --no-dereference is specified
    if args.no_dereference && path.is_symlink() {
        if args.verbose {
            println!("Setting timestamps on symlink: {}", path.display().to_string().cyan());
            println!("Warning: Symlink timestamp modification not fully supported on this platform");
        }
        return Ok(());
    }
    
    // Get current times if we only want to modify one
    let current_metadata = path.metadata()
        .with_context(|| format!("Failed to read current timestamps for {}", path.display()))?;
    
    let current_access = current_metadata.accessed()?;
    let current_modified = current_metadata.modified()?;
    
    // Use specified times or keep current ones
    let access_time = time_spec.access_time.unwrap_or(current_access);
    let modification_time = time_spec.modification_time.unwrap_or(current_modified);
    
    filetime::set_file_times(
        path,
        filetime::FileTime::from_system_time(access_time),
        filetime::FileTime::from_system_time(modification_time)
    ).with_context(|| format!("Failed to set timestamps for {}", path.display()))?;
    
    if args.verbose {
        println!("Updated timestamps for: {}", path.display().to_string().cyan());
    }
    
    Ok(())
}

/// Parse timestamp from various formats
fn parse_timestamp(args: &Args) -> Result<Option<SystemTime>> {
    // Priority: reference file > date string > timestamp format
    if let Some(ref_file) = &args.reference {
        return parse_reference_time(ref_file);
    }
    
    if let Some(date_str) = &args.date {
        return parse_date_string(date_str);
    }
    
    if let Some(timestamp_str) = &args.timestamp {
        return parse_timestamp_format(timestamp_str);
    }
    
    Ok(None)
}

/// Parse reference file timestamps
fn parse_reference_time(reference_path: &str) -> Result<Option<SystemTime>> {
    let path = Path::new(reference_path);
    if !path.exists() {
        anyhow::bail!("Reference file does not exist: {}", reference_path);
    }
    
    let metadata = path.metadata()
        .with_context(|| format!("Failed to read metadata from reference file: {}", reference_path))?;
    
    // For reference files, we use the modification time as the base
    Ok(Some(metadata.modified()?))
}

/// Parse date string like "2023-12-25 15:30:45" or "2023-12-25"
fn parse_date_string(date_str: &str) -> Result<Option<SystemTime>> {
    // Try different common formats
    let formats = [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M", 
        "%Y-%m-%d",
        "%m/%d/%Y %H:%M:%S",
        "%m/%d/%Y %H:%M",
        "%m/%d/%Y",
        "%d.%m.%Y %H:%M:%S",
        "%d.%m.%Y %H:%M",
        "%d.%m.%Y",
    ];
    
    for format in &formats {
        if let Ok(parsed) = NaiveDateTime::parse_from_str(date_str, format) {
            let dt = DateTime::<Utc>::from_naive_utc_and_offset(parsed, Utc);
            return Ok(Some(SystemTime::from(dt)));
        }
        // Try parsing as date only and add midnight
        if let Ok(parsed) = chrono::NaiveDate::parse_from_str(date_str, &format.replace(" %H:%M:%S", "").replace(" %H:%M", "")) {
            let dt = parsed.and_hms_opt(0, 0, 0).unwrap();
            let dt = DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc);
            return Ok(Some(SystemTime::from(dt)));
        }
    }
    
    anyhow::bail!("Unable to parse date string: {}", date_str);
}

/// Parse timestamp format [[CC]YY]MMDDhhmm[.ss]
fn parse_timestamp_format(timestamp_str: &str) -> Result<Option<SystemTime>> {
    // Remove optional seconds part
    let (base, seconds) = if timestamp_str.contains('.') {
        let parts: Vec<&str> = timestamp_str.split('.').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid timestamp format: {}", timestamp_str);
        }
        (parts[0], Some(parts[1].parse::<u32>()?))
    } else {
        (timestamp_str, None)
    };
    
    let base_len = base.len();
    
    // Parse based on length: 8, 10, or 12 digits
    let (year, month, day, hour, minute) = match base_len {
        8 => { // MMDDHHMM (current year assumed)
            let current_year = chrono::Utc::now().year();
            (current_year, base[0..2].parse()?, base[2..4].parse()?, base[4..6].parse()?, base[6..8].parse()?)
        },
        10 => { // YYMMDDHHMM
            let yy: i32 = base[0..2].parse()?;
            let year = if yy >= 70 { 1900 + yy } else { 2000 + yy };
            (year, base[2..4].parse()?, base[4..6].parse()?, base[6..8].parse()?, base[8..10].parse()?)
        },
        12 => { // CCYYMMDDHHMM  
            let cc: i32 = base[0..2].parse()?;
            let yy: i32 = base[2..4].parse()?;
            (cc * 100 + yy, base[4..6].parse()?, base[6..8].parse()?, base[8..10].parse()?, base[10..12].parse()?)
        },
        _ => anyhow::bail!("Invalid timestamp format length: {} (expected 8, 10, or 12 digits)", base_len)
    };
    
    let seconds = seconds.unwrap_or(0);
    
    let naive_dt = chrono::NaiveDate::from_ymd_opt(year, month, day)
        .and_then(|d| d.and_hms_opt(hour, minute, seconds))
        .ok_or_else(|| anyhow::anyhow!("Invalid timestamp values: {}-{}-{} {}:{}:{}", year, month, day, hour, minute, seconds))?;
    
    let dt = DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc);
    Ok(Some(SystemTime::from(dt)))
}

/// Determine which timestamps to set based on flags
fn get_time_spec(args: &Args, custom_time: Option<SystemTime>) -> Result<TimeSpec> {
    let now = custom_time.unwrap_or_else(SystemTime::now);
    
    let (access_time, modification_time) = if args.access_time_only {
        (Some(now), None)
    } else if args.modification_time_only {
        (None, Some(now))
    } else {
        // Default: set both times
        (Some(now), Some(now))
    };
    
    Ok(TimeSpec {
        access_time,
        modification_time,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    fn create_test_args(paths: Vec<String>) -> Args {
        Args {
            paths,
            directory: false,
            file: false,
            parents: false,
            mode: None,
            interactive: false,
            verbose: false,
            no_create: false,
            date: None,
            timestamp: None,
            reference: None,
            access_time_only: false,
            modification_time_only: false,
            no_dereference: false,
        }
    }

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

    #[test]
    fn test_date_parsing() {
        let result = parse_date_string("2023-12-25 15:30:00");
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
        
        let result = parse_date_string("2023-12-25");
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
        
        let result = parse_date_string("invalid-date");
        assert!(result.is_err());
    }

    #[test]
    fn test_timestamp_parsing() {
        let result = parse_timestamp_format("202312251530");
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
        
        let result = parse_timestamp_format("202312251530.45");
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
        
        let result = parse_timestamp_format("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_argument_validation() {
        let mut args = create_test_args(vec!["test.txt".to_string()]);
        
        // Should succeed with valid args
        assert!(validate_arguments(&args).is_ok());
        
        // Should fail with conflicting flags
        args.directory = true;
        args.file = true;
        assert!(validate_arguments(&args).is_err());
        
        // Reset and test time conflicts
        args = create_test_args(vec!["test.txt".to_string()]);
        args.access_time_only = true;
        args.modification_time_only = true;
        assert!(validate_arguments(&args).is_err());
        
        // Reset and test multiple time sources
        args = create_test_args(vec!["test.txt".to_string()]);
        args.date = Some("2023-01-01".to_string());
        args.timestamp = Some("202301011200".to_string());
        assert!(validate_arguments(&args).is_err());
    }
}