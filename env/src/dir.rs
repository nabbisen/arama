use std::{
    env,
    fs::create_dir_all,
    io::{Error, ErrorKind, Result},
    path::{Path, PathBuf},
};

const LOCAL_DIR: &str = ".arama-local";

pub fn local_dir() -> Result<PathBuf> {
    let current_exe = env::current_exe()?;
    let path = current_exe
        .parent()
        .expect("failed to get exe parent directory")
        .join(LOCAL_DIR);
    Ok(path.to_path_buf())
}

pub fn validate_dir(path: &Path) -> Result<()> {
    if !path.exists() {
        return create_dir_all(&path);
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
