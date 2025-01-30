use std::fs;
use std::path::Path;

fn main() {
    let src_path = Path::new("assets");
    let dest_path = Path::new("/usr/local/share/flyer/assets");

    // Ensure the destination directory exists
    if !dest_path.exists() {
        fs::create_dir_all(dest_path).expect("Failed to create asset directory");
    }

    // Copy assets recursively
    copy_dir_recursive(src_path, dest_path).expect("Failed to copy assets");

    println!("cargo:rerun-if-changed=assets");
}

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
