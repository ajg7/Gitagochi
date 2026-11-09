//! Creating the learner's starter project.

use crate::git::{self, GitError};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const STARTER_CARGO_TOML: &str = include_str!("../../curriculum/starter/Cargo.toml.in");
pub const STARTER_LIB: &str = include_str!("../../curriculum/starter/lib.rs");
pub const STARTER_MAIN: &str = include_str!("../../curriculum/starter/main.rs");
pub const STARTER_GITIGNORE: &str = include_str!("../../curriculum/starter/gitignore");

#[derive(Debug)]
pub enum ScaffoldError {
    NotEmpty(PathBuf),
    Io(io::Error),
    Git(GitError),
}

impl fmt::Display for ScaffoldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScaffoldError::NotEmpty(p) => write!(
                f,
                "{} already exists and is not an empty folder; choose a new or empty folder",
                p.display()
            ),
            ScaffoldError::Io(e) => write!(f, "could not create the project: {e}"),
            ScaffoldError::Git(e) => write!(f, "could not set up git: {e}"),
        }
    }
}

impl std::error::Error for ScaffoldError {}

/// Write the starter files into `dir` (new or empty) and `git init` it.
/// Refuses, writing nothing, if `dir` is a file or a non-empty folder.
pub fn create(dir: &Path) -> Result<(), ScaffoldError> {
    if dir.exists() {
        let empty = dir.is_dir()
            && fs::read_dir(dir)
                .map_err(ScaffoldError::Io)?
                .next()
                .is_none();
        if !empty {
            return Err(ScaffoldError::NotEmpty(dir.to_path_buf()));
        }
    }
    let files = [
        ("Cargo.toml", STARTER_CARGO_TOML),
        ("src/lib.rs", STARTER_LIB),
        ("src/main.rs", STARTER_MAIN),
        (".gitignore", STARTER_GITIGNORE),
    ];
    fs::create_dir_all(dir.join("src")).map_err(ScaffoldError::Io)?;
    for (name, text) in files {
        fs::write(dir.join(name), text).map_err(ScaffoldError::Io)?;
    }
    git::init(dir).map_err(ScaffoldError::Git)
}
