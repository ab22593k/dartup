use crate::config;
use crate::profile::Profile;
use anyhow::Result;

/// Path to the profile sidecar file for a given toolchain version.
fn profile_sidecar_path(version: &str) -> std::path::PathBuf {
    config::envs_dir().join(version).join(".profile")
}

/// Save the installation profile to a sidecar JSON file.
pub fn save_profile(version: &str, profile: &Profile) -> Result<()> {
    let path = profile_sidecar_path(version);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(profile)?;
    std::fs::write(&path, json)?;
    Ok(())
}

/// Load the installation profile from the sidecar JSON file.
/// Returns `None` if the sidecar file doesn't exist, is malformed, or the
/// toolchain directory doesn't exist.
pub fn load_profile(version: &str) -> Option<Profile> {
    let path = profile_sidecar_path(version);
    if !path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::Component;
    use std::collections::HashSet;
    use std::sync::atomic::{AtomicU32, Ordering};

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn tmp_version() -> String {
        let n = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        format!("test-ver-{n}")
    }

    #[test]
    fn test_save_profile_full_writes_json() {
        let ver = tmp_version();
        let path = profile_sidecar_path(&ver);
        // Ensure clean state
        std::fs::remove_file(&path).ok();

        save_profile(&ver, &Profile::Full).unwrap();
        assert!(path.exists(), "sidecar file should exist");

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(
            content.contains(r#""profile""#),
            "should have 'profile' key"
        );
        assert!(content.contains(r#""full""#), "should have 'full' value");

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_save_profile_custom_writes_components() {
        let ver = tmp_version();
        let path = profile_sidecar_path(&ver);
        std::fs::remove_file(&path).ok();

        let custom = Profile::Custom(HashSet::from([Component::Engine, Component::Android]));
        save_profile(&ver, &custom).unwrap();
        assert!(path.exists(), "sidecar file should exist");

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(
            content.contains(r#""custom""#),
            "should have 'custom' value"
        );
        assert!(content.contains(r#""engine""#), "should contain 'engine'");
        assert!(content.contains(r#""android""#), "should contain 'android'");

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_load_profile_full_reads_json() {
        let ver = tmp_version();
        let path = profile_sidecar_path(&ver);
        std::fs::remove_file(&path).ok();

        save_profile(&ver, &Profile::Full).unwrap();
        let loaded = load_profile(&ver);
        assert_eq!(loaded, Some(Profile::Full), "should load Full profile");

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_load_profile_custom_reads_components() {
        let ver = tmp_version();
        let path = profile_sidecar_path(&ver);
        std::fs::remove_file(&path).ok();

        let original = Profile::Custom(HashSet::from([Component::Engine, Component::Web]));
        save_profile(&ver, &original).unwrap();
        let loaded = load_profile(&ver).unwrap();
        assert_eq!(
            loaded, original,
            "should load Custom profile with components"
        );

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_load_profile_missing_file_returns_none() {
        let ver = tmp_version();
        let path = profile_sidecar_path(&ver);
        std::fs::remove_file(&path).ok();
        assert_eq!(load_profile(&ver), None, "missing file should return None");
    }

    #[test]
    fn test_load_profile_invalid_json_returns_none() {
        let ver = tmp_version();
        let path = profile_sidecar_path(&ver);
        std::fs::remove_file(&path).ok();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&path, b"not valid json").ok();
        assert_eq!(load_profile(&ver), None, "invalid JSON should return None");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_save_profile_default_roundtrip() {
        let ver = tmp_version();
        let path = profile_sidecar_path(&ver);
        std::fs::remove_file(&path).ok();

        save_profile(&ver, &Profile::Default).unwrap();
        assert_eq!(load_profile(&ver), Some(Profile::Default));

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_save_profile_minimal_roundtrip() {
        let ver = tmp_version();
        let path = profile_sidecar_path(&ver);
        std::fs::remove_file(&path).ok();

        save_profile(&ver, &Profile::Minimal).unwrap();
        assert_eq!(load_profile(&ver), Some(Profile::Minimal));

        std::fs::remove_file(&path).ok();
    }
}
