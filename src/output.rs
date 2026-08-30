use crate::args::Args;
use form3::ansi::Color;
use form3::compat::Colorize;
use std::path::Path;

/// Render a path in the given color. Centralizes the
/// `path.display().to_string().<color>()` pattern that was previously
/// repeated at every call site.
pub fn colored_path(path: &Path, color: Color) -> String {
    path.display().to_string().color(color).to_string()
}

/// Print a per-path progress line: full detail in verbose mode, a compact
/// checkmark when batching multiple paths, nothing otherwise. Centralizes
/// the checkmark/verbose/batch branching that was duplicated after both
/// the create path and the timestamp-only path.
pub fn report_progress(path: &Path, args: &Args, verbose_label: &str) {
    if args.verbose {
        println!("{} {} {}", "✓".bright_green(), verbose_label, colored_path(path, Color::Green));
    } else if args.paths.len() > 1 {
        println!("{} {}", "✓".bright_green(), colored_path(path, Color::Green));
    }
}
