//! Shared public-seam fixtures for the `bnfgen-syntax` integration tests.
//!
//! Fixtures live in `tests/fixtures/*.bnfgen`, one source per file. The
//! directory is discovered at runtime so adding a fixture automatically adds
//! it to the snapshot test. Files may omit a trailing newline (see
//! `.editorconfig`).

mod snapshot;

use std::fs;
use std::path::Path;

use walkdir::WalkDir;

pub use snapshot::snapshot;

/// Discover every fixture file and return its file stem with its source.
///
/// Sorting by path makes snapshot execution and failure order deterministic,
/// independent of the directory traversal order supplied by the filesystem.
pub fn fixtures() -> Vec<(String, String)> {
    let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut fixtures = WalkDir::new(&fixture_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "bnfgen"))
        .map(|entry| {
            let path = entry.into_path();
            let name = path
                .file_stem()
                .expect("fixture files must have a file stem")
                .to_string_lossy()
                .into_owned();
            let source = fs::read_to_string(&path).unwrap_or_else(|error| {
                panic!("failed to read fixture {}: {error}", path.display())
            });
            (name, source)
        })
        .collect::<Vec<_>>();

    fixtures.sort_by(|left, right| left.0.cmp(&right.0));
    fixtures
}
