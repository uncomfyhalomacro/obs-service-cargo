use std::fs;
use std::path::{Path, PathBuf};

/// Get project root. This is a hacky function that assumes that the first
/// manifest file found determines the project root.
pub fn get_project_root(srcdir: impl AsRef<Path>) -> Result<impl AsRef<Path>, ()> {
    let target_file = "Cargo.toml";
    let mut target_dir = PathBuf::from("/");
    for entry in std::fs::read_dir(srcdir).expect("Error reading directory") {
        let entry = &entry.expect("Error reading content in dir");
        let pathdir = &entry.path();
        let is_manifest_file = format!("{}/{}", pathdir.display(), target_file);
        let is_manifest_file = Path::new(&is_manifest_file);
        if is_manifest_file.exists() && is_manifest_file.is_file() {
            target_dir.push(&is_manifest_file.parent().expect("File has no parent"));
            break;
        } else {
            continue;
        }
    }
    if target_dir == PathBuf::from("/") {
        Err(())
    } else {
        Ok(target_dir)
    }
}

/// Call cargo vendor command
/// Default arg is `vendor -vv`. Additional arguments are as `args: Vec<&str>`.
pub fn cargo_vendor(srcdir: impl AsRef<Path>, args: Vec<&str>) -> std::io::Result<()> {
    println!("Vendoring deps at {}", srcdir.as_ref().display());
    let cargo_command = std::process::Command::new("cargo")
        .arg("vendor")
        .arg("-vv")
        .args(&args)
        .current_dir(&srcdir)
        .output()
        .expect("Something went wrong");
    let output = unsafe { String::from_utf8_unchecked(cargo_command.stdout) };
    if cargo_command.status.success() {
        println!("{}", &output);
        println!("Vendoring dependencies was successful");
    } else {
        eprintln!("{}", &output);
        panic!("Failed to vendor dependencies");
    }
    Ok(())
}

/// This will be used to copy directories to target destination recursively
pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

/// Most commonly used compression algorithms for Rust projects
pub enum CompressionType {
    Zstd,
    Lzma,
    Gzip,
    Zip,
}

pub fn get_compression_type(file: impl AsRef<Path>) -> Result<CompressionType, ()> {
    match &file.as_ref().extension() {
        Some(ext) => {
            let ext = ext.to_str().expect("Failed to convert to str");
            match ext {
                "zst" => Ok(CompressionType::Zstd),
                "zstd" => Ok(CompressionType::Zstd),
                "gz" => Ok(CompressionType::Gzip),
                "zip" => Ok(CompressionType::Zip),
                "xz" => Ok(CompressionType::Lzma),
                _ => Err(()),
            }
        }
        None => Err(()),
    }
}
