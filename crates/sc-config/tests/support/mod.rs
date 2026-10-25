#![allow(dead_code)]

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use sc_config::{AppName, ConfigError, Loader};
use serde_json::Value;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    pub fn new() -> Self {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let id = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        path.push(format!("sc-config-{}-{}-{}", std::process::id(), nanos, id));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub fn write_file(dir: &Path, name: &str, contents: impl AsRef<[u8]>) {
    fs::write(dir.join(name), contents).unwrap();
}

pub fn environment(pairs: &[(&str, &str)]) -> Vec<(OsString, OsString)> {
    pairs
        .iter()
        .map(|(name, value)| (OsString::from(*name), OsString::from(*value)))
        .collect()
}

pub fn load_value(dir: &Path, app: &str, pairs: &[(&str, &str)]) -> Result<Value, ConfigError> {
    Loader::new(
        AppName::new(app).unwrap(),
        dir.to_path_buf(),
        environment(pairs),
    )
    .load()
}

pub fn snapshot(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name().to_string_lossy().into_owned();
        let bytes = fs::read(entry.path()).unwrap();
        entries.push((name, bytes));
    }
    entries.sort();
    entries
}
