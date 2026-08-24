use std::{
    fs::{OpenOptions, create_dir_all},
    io::{Result, Write},
    path::Path,
};

use crate::dir::local_dir;

const DIAGNOSTIC_FILE: &str = "diagnostic.log";

/// RFC 041 §7 requires resolved data locations - and, by the same
/// reasoning, other startup/runtime failures - to be "logged or
/// otherwise discoverable, so a failing run can be diagnosed without a
/// debugger." A Windows release build (Task 037's `windows_subsystem`
/// attribute) has no console for `eprintln!` to reach, so on exactly that
/// configuration this appends the message to a file under [`local_dir`]
/// instead - the same directory RFC 042 phase two already resolved this
/// question for (Task 026: a packaged app's shell activation has no
/// console either, and writes to the package's own writable location).
/// Everywhere else - debug builds, Linux, macOS - stderr is exactly as
/// discoverable as it always was, so it is left alone.
pub fn diagnostic(message: &str) {
    if cfg!(all(not(debug_assertions), target_os = "windows")) {
        let _ = append_to_file(message);
    } else {
        eprintln!("{message}");
    }
}

fn append_to_file(message: &str) -> Result<()> {
    append_to_path(&local_dir()?.join(DIAGNOSTIC_FILE), message)
}

/// The pure half of [`append_to_file`] - same seam as
/// [`crate::dir::local_dir_with_override`]: real callers resolve the path
/// via [`local_dir`], tests supply an explicit one directly, so this can be
/// proven without touching the real platform data directory.
fn append_to_path(path: &Path, message: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{message}")
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::append_to_path;

    #[test]
    fn append_to_path_creates_the_file_and_its_parent_directory() {
        let dir = std::env::temp_dir().join(format!(
            "arama-diagnostic-test-{}-fresh",
            std::process::id()
        ));
        let path = dir.join("diagnostic.log");
        let _ = fs::remove_dir_all(&dir);

        append_to_path(&path, "first line").unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "first line\n");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn append_to_path_adds_to_an_existing_file_rather_than_overwriting_it() {
        let dir = std::env::temp_dir().join(format!(
            "arama-diagnostic-test-{}-append",
            std::process::id()
        ));
        let path = dir.join("diagnostic.log");
        let _ = fs::remove_dir_all(&dir);

        append_to_path(&path, "first line").unwrap();
        append_to_path(&path, "second line").unwrap();

        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "first line\nsecond line\n"
        );
        fs::remove_dir_all(&dir).unwrap();
    }
}
