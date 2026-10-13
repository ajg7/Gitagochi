//! Persistence: one JSON file per repo path under the platform data dir.

use crate::pet::Pet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Stable FNV-1a hash (std's hasher is not stable across releases).
fn fnv1a(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// Directory holding all save files.
pub fn default_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("gitagochi")
}

/// Save file path for a repo, inside `data_dir`.
pub fn save_path(data_dir: &Path, repo: &Path) -> PathBuf {
    let key = repo.to_string_lossy().to_lowercase();
    let name = repo
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let name: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();
    data_dir.join(format!("{name}-{:016x}.json", fnv1a(&key)))
}

/// Load the pet. Missing file -> fresh pet. Corrupt file -> fresh pet and the
/// bad file is renamed to `<file>.bak`. Never panics.
pub fn load(path: &Path, now: i64) -> Pet {
    let text = match fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Pet::new(now),
        Err(_) => {
            backup(path);
            return Pet::new(now);
        }
    };
    match serde_json::from_str::<Pet>(&text) {
        Ok(mut pet) => {
            pet.sanitize();
            pet
        }
        Err(_) => {
            backup(path);
            Pet::new(now)
        }
    }
}

fn backup(path: &Path) {
    let mut bak = path.as_os_str().to_owned();
    bak.push(".bak");
    let bak = PathBuf::from(bak);
    let _ = fs::remove_file(&bak);
    let _ = fs::rename(path, &bak);
}

/// Save atomically (write temp file, then rename).
pub fn save(path: &Path, pet: &Pet) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let json = serde_json::to_string_pretty(pet).map_err(io::Error::other)?;
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    fs::write(&tmp, json)?;
    fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pet::FedCommit;

    #[test]
    fn round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sub").join("pet.json");
        let mut pet = Pet::new(5);
        pet.feed(&[FedCommit {
            id: "abc".into(),
            summary: "hi".into(),
            time: 1,
        }]);
        pet.last_seen = Some("abc".into());
        save(&path, &pet).unwrap();
        assert_eq!(load(&path, 99), pet);
    }

    #[test]
    fn missing_file_gives_fresh_pet() {
        let dir = tempfile::tempdir().unwrap();
        let pet = load(&dir.path().join("none.json"), 42);
        assert_eq!(pet, Pet::new(42));
    }

    #[test]
    fn corrupt_file_gives_fresh_pet_and_bak() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("pet.json");
        fs::write(&path, "{ not json").unwrap();
        let pet = load(&path, 7);
        assert_eq!(pet, Pet::new(7));
        assert!(!path.exists());
        let bak = dir.path().join("pet.json.bak");
        assert_eq!(fs::read_to_string(bak).unwrap(), "{ not json");
    }

    #[test]
    fn save_paths_differ_per_repo() {
        let d = Path::new("data");
        let a = save_path(d, Path::new("/x/repo"));
        let b = save_path(d, Path::new("/y/repo"));
        assert_ne!(a, b);
        assert_eq!(a, save_path(d, Path::new("/x/repo")));
    }
}
