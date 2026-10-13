//! Persistence: one shared pet save plus legacy per-repo save migration.

use crate::pet::{Pet, RepoTrack};
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

/// The shared pet save file, independent of which repos are selected.
pub fn shared_path(data_dir: &Path) -> PathBuf {
    data_dir.join("pet.json")
}

/// The repo selection config file.
pub fn config_path(data_dir: &Path) -> PathBuf {
    data_dir.join("repos.json")
}

/// Legacy per-repo save file path for a repo, inside `data_dir`.
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

fn is_legacy_name(name: &str) -> bool {
    let Some(stem) = name.strip_suffix(".json") else {
        return false;
    };
    match stem.rsplit_once('-') {
        Some((_, h)) => h.len() == 16 && h.bytes().all(|b| b.is_ascii_hexdigit()),
        None => false,
    }
}

/// Adopt the most recently modified legacy per-repo save as the shared pet.
/// If it belonged to one of `repo_keys`, that repo's tracking is carried over.
/// Does nothing (returns `None`) if a shared save or its backup exists.
pub fn migrate_legacy(data_dir: &Path, repo_keys: &[String], now: i64) -> Option<Pet> {
    let shared = shared_path(data_dir);
    let mut bak = shared.as_os_str().to_owned();
    bak.push(".bak");
    if shared.exists() || Path::new(&bak).exists() {
        return None;
    }
    let mut files: Vec<(std::time::SystemTime, PathBuf)> = fs::read_dir(data_dir)
        .ok()?
        .flatten()
        .filter(|e| is_legacy_name(&e.file_name().to_string_lossy()))
        .filter_map(|e| Some((e.metadata().ok()?.modified().ok()?, e.path())))
        .collect();
    files.sort_by(|a, b| b.cmp(a));
    for (_, path) in files {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let Some(v) = serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .filter(serde_json::Value::is_object)
        else {
            continue;
        };
        let mut v = v;
        if v.get("last_update").is_none() {
            v["last_update"] = now.into();
        }
        let Ok(mut pet) = serde_json::from_value::<Pet>(v.clone()) else {
            continue;
        };
        if let Some(key) = repo_keys
            .iter()
            .find(|k| save_path(data_dir, Path::new(k)) == path)
        {
            let mut t = RepoTrack {
                initialized: v["initialized"].as_bool().unwrap_or(false),
                last_seen: v["last_seen"].as_str().map(str::to_string),
                fed_ids: serde_json::from_value(v["fed_ids"].clone()).unwrap_or_default(),
            };
            let recent: Vec<String> = pet.recent.iter().rev().map(|r| r.id.clone()).collect();
            for id in recent {
                if !t.has_fed(&id) {
                    t.fed_ids.push(id);
                }
            }
            pet.repos.insert(key.clone(), t);
        }
        pet.sanitize();
        return Some(pet);
    }
    None
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
    let parsed = serde_json::from_str::<serde_json::Value>(&text)
        .ok()
        .filter(serde_json::Value::is_object)
        .map(|mut v| {
            if v.get("last_update").is_none() {
                v["last_update"] = now.into();
            }
            v
        })
        .and_then(|v| serde_json::from_value::<Pet>(v).ok())
        .ok_or(());
    match parsed {
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
            repo: String::new(),
        }]);
        pet.repos.entry("r".into()).or_default().last_seen = Some("abc".into());
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
