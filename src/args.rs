use anyhow::Result;
use clap::Parser;

/// Bank: A comprehensive command-line utility combining mkdir, touch, and advanced filesystem operations
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The paths to create (files or directories)
    #[arg(value_name = "PATH", required = true)]
    pub paths: Vec<String>,

    /// Force creation as directory (mkdir mode)
    #[arg(short = 'd', long = "directory")]
    pub directory: bool,

    /// Force creation as file (touch mode)
    #[arg(short = 'f', long = "file")]
    pub file: bool,

    /// Create parent directories as needed
    #[arg(short = 'p', long = "parents")]
    pub parents: bool,

    /// Set file/directory permissions (octal format, e.g., 755)
    #[arg(short = 'm', long = "mode")]
    pub mode: Option<String>,

    /// Interactive mode for ambiguous paths
    #[arg(short = 'i', long = "interactive")]
    pub interactive: bool,

    /// Verbose output
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// Do not create files, only update timestamps if they exist
    #[arg(short = 'c', long = "no-create")]
    pub no_create: bool,

    /// Parse date string and use it instead of current time
    #[arg(long = "date", value_name = "STRING")]
    pub date: Option<String>,

    /// Use timestamp format [[CC]YY]MMDDhhmm[.ss] instead of current time
    #[arg(short = 't', long = "timestamp", value_name = "STAMP")]
    pub timestamp: Option<String>,

    /// Use this file's times instead of current time
    #[arg(short = 'r', long = "reference", value_name = "FILE")]
    pub reference: Option<String>,

    /// Change only the access time
    #[arg(short = 'a', long = "atime")]
    pub access_time_only: bool,

    /// Change only the modification time
    #[arg(long = "mtime")]
    pub modification_time_only: bool,

    /// Affect symbolic links instead of referenced files
    #[arg(long = "no-dereference")]
    pub no_dereference: bool,
}

/// Validate argument combinations
pub fn validate_arguments(args: &Args) -> Result<()> {
    if args.directory && args.file {
        anyhow::bail!("Cannot specify both --directory and --file flags");
    }

    let time_sources = [args.date.is_some(), args.timestamp.is_some(), args.reference.is_some()];
    let time_source_count = time_sources.iter().filter(|&&x| x).count();
    if time_source_count > 1 {
        anyhow::bail!("Cannot specify multiple time sources (--date, --timestamp, --reference)");
    }

    if args.access_time_only && args.modification_time_only {
        anyhow::bail!("Cannot specify both --atime and --mtime flags");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::create_test_args;

    #[test]
    fn test_argument_validation() {
        let mut args = create_test_args(vec!["test.txt".to_string()]);
        assert!(validate_arguments(&args).is_ok());

        args.directory = true;
        args.file = true;
        assert!(validate_arguments(&args).is_err());

        args = create_test_args(vec!["test.txt".to_string()]);
        args.access_time_only = true;
        args.modification_time_only = true;
        assert!(validate_arguments(&args).is_err());

        args = create_test_args(vec!["test.txt".to_string()]);
        args.date = Some("2023-01-01".to_string());
        args.timestamp = Some("202301011200".to_string());
        assert!(validate_arguments(&args).is_err());
    }
}
