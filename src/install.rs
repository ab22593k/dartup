use crate::config;
use crate::releases;
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::io::{BufWriter, Read};
use std::path::Path;

/// Download a file with a progress bar
fn download_with_progress(url: &str, dest: &Path) -> Result<()> {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(url)
        .send()
        .context(format!("Failed to start download from {url}"))?;

    let total_size = resp.content_length().unwrap_or(0);

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message(format!(
        "Downloading {}",
        url.split('/').next_back().unwrap_or(url)
    ));

    let mut dest_file = BufWriter::new(File::create(dest)?);
    let mut source = resp.take(total_size.max(1));

    let mut downloaded: u64 = 0;
    let mut buffer = [0u8; 8192];
    loop {
        let n = std::io::Read::read(&mut source, &mut buffer)?;
        if n == 0 {
            break;
        }
        std::io::Write::write_all(&mut dest_file, &buffer[..n])?;
        downloaded += n as u64;
        pb.set_position(downloaded);
    }

    pb.finish_with_message(format!(
        "Downloaded {}",
        url.split('/').next_back().unwrap_or(url)
    ));
    Ok(())
}

/// Extract a .tar.xz archive
fn extract_tar_xz(archive: &Path, dest: &Path) -> Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Extracting Flutter SDK...");

    let file = File::open(archive)?;
    let decoder = xz2::read::XzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);
    archive.unpack(dest)?;

    pb.finish_with_message("Extracted Flutter SDK");
    Ok(())
}

/// Extract a .zip archive
fn extract_zip(archive: &Path, dest: &Path) -> Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Extracting Flutter SDK...");

    let file = File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file)?;
    zip.extract(dest)?;

    pb.finish_with_message("Extracted Flutter SDK");
    Ok(())
}

/// Determine the extraction type from the archive URL or path
fn extract_archive(archive: &Path, dest: &Path) -> Result<()> {
    let name = archive.to_string_lossy();
    if name.ends_with(".tar.xz") {
        extract_tar_xz(archive, dest)
    } else if name.ends_with(".zip") {
        extract_zip(archive, dest)
    } else {
        anyhow::bail!("Unsupported archive format: {name}")
    }
}

/// Install a specific Flutter version
pub fn install_version(version: &str, force: bool) -> Result<()> {
    let env_dir = config::envs_dir().join(version);

    // Check if already installed
    if env_dir.join("bin").join("flutter").exists()
        || env_dir.join("bin").join("flutter.bat").exists()
    {
        if !force {
            println!("✅ Version {version} is already installed. Use --force to reinstall.");
            return Ok(());
        }
        println!("♻️  Reinstalling {version}...");
        std::fs::remove_dir_all(&env_dir)?;
    }

    // Find the release info
    let release = releases::find_release(version)?;
    let download_url = &release.archive_url;

    println!("📦 Installing Flutter {version} ({})", release.channel);

    // Create temp directory for download
    let tmp_dir = config::dartup_home().join(".tmp");
    std::fs::create_dir_all(&tmp_dir)?;

    let archive_name = download_url
        .split('/')
        .next_back()
        .unwrap_or("flutter.tar.xz");
    let archive_path = tmp_dir.join(archive_name);

    // Download
    download_with_progress(download_url, &archive_path)?;

    // Extract
    std::fs::create_dir_all(&env_dir)?;
    extract_archive(&archive_path, &env_dir)?;

    // Find the extracted flutter directory (archives contain a flutter/ or flutter_*/ directory)
    let extracted = std::fs::read_dir(&env_dir)?
        .filter_map(|e| e.ok())
        .find(|e| e.file_name().to_string_lossy().contains("flutter"))
        .map(|e| e.path())
        .unwrap_or_else(|| {
            // If extraction didn't create a subfolder, the env_dir IS the SDK
            env_dir.clone()
        });

    // If the SDK was extracted to a subdirectory, move contents up
    if extracted != env_dir {
        for entry in std::fs::read_dir(&extracted)? {
            let entry = entry?;
            let dest = env_dir.join(entry.file_name());
            if dest.exists() {
                std::fs::remove_dir_all(&dest).ok();
            }
            std::fs::rename(entry.path(), &dest)?;
        }
        std::fs::remove_dir_all(&extracted)?;
    }

    // Cleanup archive
    std::fs::remove_file(&archive_path)?;

    println!(
        "✅ Flutter {version} installed successfully at {}",
        env_dir.display()
    );
    Ok(())
}
