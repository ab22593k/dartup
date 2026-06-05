use crate::config;
use crate::util::{dir_size, human_size};
use anyhow::Result;
use colored::Colorize;

/// Run garbage collection on cached artifacts
pub fn run_gc() -> Result<()> {
    println!("{}", "Running garbage collection...".bold());

    let engine_cache = config::engine_cache_dir();
    let git_cache = config::git_cache_dir();

    // Find which engine versions are actually in use by installed environments
    let used_engines = find_used_engine_versions()?;

    // Clean unused engine artifacts
    if engine_cache.exists() {
        let mut cleaned = 0u64;
        for entry in std::fs::read_dir(&engine_cache)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if !used_engines.contains(&name) {
                let size = dir_size(entry.path());
                if entry.path().is_dir() {
                    std::fs::remove_dir_all(entry.path())?;
                } else {
                    std::fs::remove_file(entry.path())?;
                }
                cleaned += size;
                println!(
                    "  🗑️  Removed unused engine cache: {} ({})",
                    name,
                    human_size(size)
                );
            }
        }
        if cleaned > 0 {
            println!("✅ Freed {}", human_size(cleaned).green().bold());
        } else {
            println!("✅ No unused engine artifacts found.");
        }
    } else {
        println!("ℹ️  No engine cache directory.");
    }

    // Report git cache size
    if git_cache.exists() {
        let git_size = dir_size(git_cache);
        println!(
            "📦 Git cache: {} (shared across all versions)",
            human_size(git_size)
        );
    }

    Ok(())
}

/// Find engine version strings referenced by installed Flutter versions
fn find_used_engine_versions() -> Result<Vec<String>> {
    let envs_dir = config::envs_dir();
    let mut used = Vec::new();

    if !envs_dir.exists() {
        return Ok(used);
    }

    for entry in std::fs::read_dir(&envs_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            // Flutter stores engine version in bin/internal/engine.version
            let engine_version_file = path.join("bin").join("internal").join("engine.version");
            if engine_version_file.exists()
                && let Ok(content) = std::fs::read_to_string(engine_version_file)
            {
                let ver = content.trim().to_string();
                if !ver.is_empty() && !used.contains(&ver) {
                    used.push(ver);
                }
            }
        }
    }

    Ok(used)
}
