use crate::args::Args;

/// Shared test fixture for `Args` so each module's tests don't redefine it.
pub fn create_test_args(paths: Vec<String>) -> Args {
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
