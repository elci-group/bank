use anyhow::{Context, Result};
use form3::compat::Colorize;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub fn set_permissions(path: &Path, mode_str: &str, verbose: bool) -> Result<()> {
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
