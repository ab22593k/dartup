use crate::config::{self, PROJECT_CONFIG_FILE, ProjectConfig};
use anyhow::{Context, Result};
use colored::Colorize;
use std::path::PathBuf;

/// Find the project root by looking for .dartup.json upward from cwd
fn find_project_config() -> Result<Option<PathBuf>> {
    let cwd = std::env::current_dir()?;
    let mut dir = Some(cwd.as_path());

    while let Some(current) = dir {
        let config_path = current.join(PROJECT_CONFIG_FILE);
        if config_path.exists() {
            return Ok(Some(config_path));
        }
        dir = current.parent();
    }

    Ok(None)
}

/// Read the project version if a .dartup.json exists
pub fn read_project_version() -> Result<Option<String>> {
    if let Some(config_path) = find_project_config()? {
        let content =
            std::fs::read_to_string(&config_path).context("Failed to read .dartup.json")?;
        let config: ProjectConfig =
            serde_json::from_str(&content).context("Failed to parse .dartup.json")?;
        return Ok(Some(config.version));
    }
    Ok(None)
}

/// Check if the project version is installed, and suggest install if not
#[allow(dead_code)]
pub fn ensure_project_version() -> Result<()> {
    if let Some(version) = read_project_version()? {
        let env_dir = config::envs_dir().join(&version);
        if !env_dir.exists() {
            println!(
                "⚠️  Project requires Flutter {version} which is not installed.",
                version = version.yellow()
            );
            println!("   Run: dartup install {version}", version = version);
        }
    }
    Ok(())
}

/// Set the Flutter version for the current project
pub fn set_project_version(version: &str) -> Result<()> {
    let env_dir = config::envs_dir().join(version);
    if !env_dir.join("bin").join("flutter").exists()
        && !env_dir.join("bin").join("flutter.bat").exists()
    {
        anyhow::bail!("Flutter {version} is not installed. Run 'dartup install {version}' first.");
    }

    let config = ProjectConfig {
        version: version.to_string(),
    };

    let cwd = std::env::current_dir()?;
    let config_path = cwd.join(PROJECT_CONFIG_FILE);

    let json = serde_json::to_string_pretty(&config)?;
    std::fs::write(&config_path, json).context("Failed to write .dartup.json")?;

    println!(
        "✅ Set Flutter {} for this project (saved to .dartup.json)",
        version.green().bold()
    );
    println!("   Add .dartup.json to .gitignore to avoid committing it.");

    // Also try to update IDE configs
    try_update_vscode_config(version)?;

    Ok(())
}

/// Try to configure VS Code to use the correct Flutter SDK
fn try_update_vscode_config(version: &str) -> Result<()> {
    let env_dir = config::envs_dir().join(version);
    let flutter_bin = env_dir.join("bin");

    let cwd = std::env::current_dir()?;
    let vscode_dir = cwd.join(".vscode");
    let settings_path = vscode_dir.join("settings.json");

    if vscode_dir.exists() {
        // Read existing settings or create new
        let mut settings: serde_json::Value = if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)?;
            serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        if let Some(obj) = settings.as_object_mut() {
            obj.insert(
                "dart.flutterSdkPath".to_string(),
                serde_json::Value::String(flutter_bin.to_string_lossy().to_string()),
            );
        }

        std::fs::create_dir_all(&vscode_dir)?;
        std::fs::write(&settings_path, serde_json::to_string_pretty(&settings)?)?;
        println!("   Updated VS Code settings to use this Flutter SDK.");
    }

    Ok(())
}
