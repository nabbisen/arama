use std::{
    fs::create_dir,
    io::{Error, ErrorKind, Result},
    path::Path,
};

pub fn validate_dir(path: &Path) -> Result<()> {
    if !path.exists() {
        return create_dir(&path);
    }

    if !path.is_dir() {
        return Err(Error::new(
            ErrorKind::NotADirectory,
            format!(
                "Can't treat cache directory, bacause invalid file is found: {}",
                path.to_string_lossy(),
            )
            .as_str(),
        ));
    }

    Ok(())
}
