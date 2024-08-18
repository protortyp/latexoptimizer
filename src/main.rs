use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Update,
    Switch,
}

const HIDDEN_FOLDER: &str = ".latexoptimizer";
const PLACEHOLDER_IMAGE: &str = "placeholder.png";

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => init(),
        Commands::Update => update(),
        Commands::Switch => switch(),
    }
}

fn init() -> Result<()> {
    create_hidden_folder()?;
    create_placeholder_image()?;
    move_images_to_hidden_folder()?;
    create_symlinks()?;
    println!("Initialization complete.");
    Ok(())
}

fn update() -> Result<()> {
    move_images_to_hidden_folder()?;
    create_symlinks()?;
    println!("Update complete.");
    Ok(())
}

fn switch() -> Result<()> {
    let hidden_folder = Path::new(HIDDEN_FOLDER);
    if hidden_folder.exists() {
        remove_symlinks()?;
        restore_original_images()?;
        println!("Switched to original images.");
    } else {
        create_symlinks()?;
        println!("Switched to placeholder images.");
    }
    Ok(())
}

fn create_hidden_folder() -> Result<()> {
    fs::create_dir_all(HIDDEN_FOLDER).context("Failed to create hidden folder")
}

fn create_placeholder_image() -> Result<()> {
    let placeholder_path = Path::new(HIDDEN_FOLDER).join(PLACEHOLDER_IMAGE);
    if !placeholder_path.exists() {
        image::ImageBuffer::from_fn(100, 100, |_, _| image::Rgb([200u8, 200u8, 200u8]))
            .save(&placeholder_path)
            .context("Failed to create placeholder image")?;
    }
    Ok(())
}

fn move_images_to_hidden_folder() -> Result<()> {
    for entry in WalkDir::new(".").into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && is_image_file(path) {
            let file_name = path.file_name().unwrap();
            let dest_path = Path::new(HIDDEN_FOLDER).join(file_name);
            if !dest_path.exists() {
                fs::rename(path, &dest_path)
                    .with_context(|| format!("Failed to move {}", path.display()))?;
            }
        }
    }
    Ok(())
}

fn create_symlinks() -> Result<()> {
    for entry in WalkDir::new(HIDDEN_FOLDER)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && is_image_file(path) && path.file_name().unwrap() != PLACEHOLDER_IMAGE {
            let link_path = Path::new(".").join(path.file_name().unwrap());
            if !link_path.exists() {
                #[cfg(unix)]
                std::os::unix::fs::symlink(
                    Path::new(HIDDEN_FOLDER).join(PLACEHOLDER_IMAGE),
                    &link_path,
                )
                .with_context(|| format!("Failed to create symlink for {}", path.display()))?;

                #[cfg(windows)]
                std::os::windows::fs::symlink_file(
                    Path::new(HIDDEN_FOLDER).join(PLACEHOLDER_IMAGE),
                    &link_path,
                )
                .with_context(|| format!("Failed to create symlink for {}", path.display()))?;
            }
        }
    }
    Ok(())
}

fn remove_symlinks() -> Result<()> {
    for entry in WalkDir::new(".").into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_symlink() && is_image_file(path) {
            fs::remove_file(path)
                .with_context(|| format!("Failed to remove symlink {}", path.display()))?;
        }
    }
    Ok(())
}

fn restore_original_images() -> Result<()> {
    for entry in WalkDir::new(HIDDEN_FOLDER)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && is_image_file(path) && path.file_name().unwrap() != PLACEHOLDER_IMAGE {
            let dest_path = Path::new(".").join(path.file_name().unwrap());
            fs::rename(path, &dest_path)
                .with_context(|| format!("Failed to restore {}", path.display()))?;
        }
    }
    Ok(())
}

fn is_image_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|s| s.to_str()),
        Some("png" | "jpg" | "jpeg" | "gif" | "bmp")
    )
}
