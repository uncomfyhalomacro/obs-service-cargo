use crate::cli::Compression;
use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display};
use std::fs;
use std::path::{Path, PathBuf};

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

pub fn cargo_vendor(srcdir: impl AsRef<Path>) -> std::io::Result<()> {
    println!("Vendoring deps at {}", srcdir.as_ref().display());
    let cargo_command = std::process::Command::new("cargo")
        .arg("-vvv")
        .arg("vendor")
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

#[derive(Debug)]
pub struct UnsupportedExtError {
    pub ext: Option<String>,
}

impl fmt::Display for UnsupportedExtError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match &self.ext {
            None => "No extension found for file. Please check if file has an extension or if it is actually a file.".to_string(),
            Some(err) => format!("{} is unsupported. If you think this is incorrect, please open an issue at https://github.com/uncomfyhalomacro/obs-service-cargo_vendor-rs/issues.", err)
        };
        write!(f, "{}", &msg)
    }
}

impl Error for UnsupportedExtError {}

pub fn get_compression_type(file: &Path) -> Result<Compression, UnsupportedExtError> {
    match file.extension() {
        Some(ext) => match ext.to_str().map(|s| s.to_string()) {
            Some(s) => match s.as_str() {
                "zst" => Ok(Compression::Zst),
                "zstd" => Ok(Compression::Zst),
                "gz" => Ok(Compression::Gz),
                "xz" => Ok(Compression::Xz),
                _ => Err(UnsupportedExtError {
                    ext: Some(s.to_string()),
                }),
            },
            None => Err(UnsupportedExtError { ext: None }),
        },
        None => Err(UnsupportedExtError { ext: None }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported_extensions() {
        let unsupported_exts = vec![
            Path::new("/uwu.txt"),
            Path::new("muwu.mi"),
            Path::new("uwu.zip"),
        ];
        for someext in unsupported_exts {
            assert_eq!(true, get_compression_type(someext).is_err());
        }
    }

    #[test]
    fn supported_extensions() {
        let supported_exts = vec![
            Path::new("uwu.tar.xz"),
            Path::new("uwu.tar.zst"),
            Path::new("uwu.tar.zstd"),
            Path::new("uwu.tar.gz"),
        ];
        for someext in supported_exts {
            assert_eq!(true, get_compression_type(someext).is_ok());
        }
    }
}
