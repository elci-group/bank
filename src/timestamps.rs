use crate::args::Args;
use crate::output::colored_path;
use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, NaiveDateTime, Utc};
use form3::ansi::Color;
use std::path::Path;
use std::time::SystemTime;

#[derive(Debug)]
pub struct TimeSpec {
    pub access_time: Option<SystemTime>,
    pub modification_time: Option<SystemTime>,
}

/// Set file timestamps with symlink handling support
pub fn set_file_times(path: &Path, time_spec: &TimeSpec, args: &Args) -> Result<()> {
    // Handle symlinks if --no-dereference is specified
    if args.no_dereference && path.is_symlink() {
        if args.verbose {
            println!("Setting timestamps on symlink: {}", colored_path(path, Color::Cyan));
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
        filetime::FileTime::from_system_time(modification_time),
    ).with_context(|| format!("Failed to set timestamps for {}", path.display()))?;

    if args.verbose {
        println!("Updated timestamps for: {}", colored_path(path, Color::Cyan));
    }

    Ok(())
}

/// Parse timestamp from various formats
pub fn parse_timestamp(args: &Args) -> Result<Option<SystemTime>> {
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
            let dt = parsed
                .and_hms_opt(0, 0, 0)
                .ok_or_else(|| anyhow::anyhow!("Unable to apply midnight time to parsed date: {}", date_str))?;
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
pub fn get_time_spec(args: &Args, custom_time: Option<SystemTime>) -> Result<TimeSpec> {
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
}
