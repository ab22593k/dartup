use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Returns the dartup home directory (~/.dartup)
pub fn dartup_home() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".dartup")
}

/// Directory where Flutter SDK versions are installed
pub fn envs_dir() -> PathBuf {
    dartup_home().join("envs")
}

/// Directory for shared engine artifact cache
pub fn engine_cache_dir() -> PathBuf {
    dartup_home().join("cache").join("engine")
}

/// Directory for shared git data (bare repo cache)
pub fn git_cache_dir() -> PathBuf {
    dartup_home().join("cache").join("git")
}

/// Path to the global default symlink
pub fn global_default_path() -> PathBuf {
    dartup_home().join("default")
}

/// Per-project config file name
pub const PROJECT_CONFIG_FILE: &str = ".dartup.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReleaseInfo {
    pub version: String,
    pub channel: String,
    pub archive_url: String,
    pub sha256: String,
    pub release_date: String,
}
