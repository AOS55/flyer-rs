use std::{fs, path::{Path, PathBuf}};
use dirs::data_local_dir;

fn main() {
    let src_path = Path::new("assets");
    let dest_path = get_asset_install_path();

    println!("Installing assets to: {}", dest_path.display());

    // Ensure the destination directory exists
    if !dest_path.exists() {
        fs::create_dir_all(&dest_path).expect("Failed to create asset directory");
    }

    // Copy assets
    copy_dir_recursive(src_path, &dest_path).expect("Failed to copy assets");

    println!("cargo:rerun-if-changed=assets");
}

/// Get the correct installation path for assets
fn get_asset_install_path() -> PathBuf {
    data_local_dir()
        .expect("Failed to get user data directory")
        .join("flyer/assets")
}

/// Recursively copies a directory
fn copy_dir_recursive(src: &Path, dest: &Path) -> std::io::Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if path.is_dir() {
            fs::create_dir_all(&dest_path)?;
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}