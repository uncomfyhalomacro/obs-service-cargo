pub mod compress;
pub mod decompress;

use crate::cli::{Compression, Opts};
use infer;
use std::error::Error;
use std::fmt;
use std::fmt::Debug;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;
use tracing::{debug, error, info, warn, Level};

const XZ_EXTS: &[&str] = &["xz"];
const ZST_EXTS: &[&str] = &["zstd", "zst"];
const GZ_EXTS: &[&str] = &["gz", "gzip"];
const XZ_MIME: &str = "application/x-xz";
const ZST_MIME: &str = "application/zstd";
const GZ_MIME: &str = "application/gzip";
const SUPPORTED_MIME_TYPES: &[&str] = &[XZ_MIME, ZST_MIME, GZ_MIME];

pub fn get_manifest_file(srcdir: impl AsRef<Path>) -> Result<PathBuf, io::Error> {
    let target_file = "Cargo.toml";

    for entry in std::fs::read_dir(srcdir).expect("Error reading directory") {
        let mut dir = entry?.path().to_owned();
        dir.push(target_file);
        if dir.exists() && dir.is_file() {
            return Ok(dir);
        } else {
            continue;
        }
    }
    Err(io::Error::new(
        io::ErrorKind::Other,
        "Not able to determine project root",
    ))
}

pub fn vendor(opts: impl AsRef<Opts>, prjdir: impl AsRef<Path>) -> Result<(), io::Error> {
    let mut prjdir = prjdir.as_ref().to_path_buf();
    let update = &opts.as_ref().update;
    let mut outdir = opts.as_ref().outdir.to_owned();
    let cargo_config = outdir.join("cargo_config");

    if *update {
        info!("Updating dependencies before vendor");
        let cargo_update = process::Command::new("cargo")
            .arg("update")
            .arg("-vv")
            .current_dir(&prjdir)
            .output()
            .expect("Failed to run cargo update.");
        if !cargo_update.status.success() {
            error!("Failed to run cargo update:\n{}", unsafe {
                String::from_utf8_unchecked(cargo_update.stderr)
            });
        } else {
            info!("Successfully ran cargo update ❤️");
        }
    } else {
        warn!("Disabled update of dependencies. You may reenable it for security updates.");
    };

    let cargo_vendor = process::Command::new("cargo")
        .arg("vendor")
        .arg("-vv")
        .current_dir(&prjdir)
        .output()
        .expect("Failed to run cargo vendor");

    if !cargo_vendor.status.success() {
        error!("Failed to run cargo vendor:\n{}", unsafe {
            std::str::from_utf8_unchecked(&cargo_vendor.stderr)
        });
    } else {
        info!(
            "Generated cargo config from vendor with content:

```
{}
```
",
            unsafe { std::str::from_utf8_unchecked(&cargo_vendor.stdout) }
        );
        fs::write(cargo_config, cargo_vendor.stdout)?;
    };

    info!("Proceeding to create compressed archive of vendored deps...");
    prjdir.push("vendor/");
    let compression: &Compression = &opts.as_ref().compression;
    debug!("Compression is of {}", &compression);
    match compression {
        Compression::Gz => {
            outdir.push("vendor.tar.gz");
            debug!("Compressed to {}", outdir.to_string_lossy());
            compress::targz("vendor", outdir, &prjdir)?
        }
        Compression::Xz => {
            outdir.push("vendor.tar.xz");
            debug!("Compressed to {}", outdir.to_string_lossy());
            compress::tarxz("vendor", outdir, &prjdir)?
        }
        Compression::Zst => {
            outdir.push("vendor.tar.zst");
            debug!("Compressed to {}", outdir.to_string_lossy());
            compress::tarzst("vendor", outdir, &prjdir)?
        }
    };
    info!("Finished creating {} compressed tarball", compression);
    Ok(())
}

pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<(), io::Error> {
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
    if file.is_file() {
        let info = infer::get_from_path(&file).expect("File is known");
        let extension = match file.extension() {
            Some(ext) => match ext.to_str() {
                Some(s) => s,
                None => "unknown extension"
            },
            None => "unknown extension"
        };
        let mimetype = match info {
            Some(ext) => ext.mime_type(),
            None => "unknown mime type",
        };
        if !SUPPORTED_MIME_TYPES.contains(&mimetype) {
            return Err(UnsupportedExtError {
                ext: Some(mimetype.to_string()),
            });
        } else {
            match mimetype {
                XZ_MIME => {
                    if XZ_EXTS.contains(&extension) {
                        info!("File has the correct supported extension {}", extension);
                    } else {
                        warn!("File has an incorrect extension: {}. Make sure it's the right compression AND extension to avoid confusion", extension);
                    };
                    Ok(Compression::Xz)
                }
                GZ_MIME => {
                    if GZ_EXTS.contains(&extension) {
                        info!("File has the correct supported extension {}", extension);
                    } else {
                        warn!("File has an incorrect extension: {}. Make sure it's the right compression AND extension to avoid confusion", extension);
                    };
                    Ok(Compression::Gz)
                }
                ZST_MIME => {
                    if ZST_EXTS.contains(&extension) {
                        info!("File has the correct supported extension {}", extension);
                    } else {
                        warn!("File has an incorrect extension: {}. Make sure it's the right compression AND extension to avoid confusion", extension);
                    };
                    Ok(Compression::Zst)
                }
                _ => unreachable!(),
            }
        }
    } else {
        error!("This is a directory!");
        Err(UnsupportedExtError {
            ext: Some("Directory".to_string()),
        })
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn unsupported_extensions() {
//         let unsupported_exts = vec![
//             Path::new("/uwu.txt"),
//             Path::new("muwu.mi"),
//             Path::new("uwu.zip"),
//         ];
//         for someext in unsupported_exts {
//             assert_eq!(true, get_compression_type(someext).is_err());
//         }
//     }

//     #[test]
//     fn supported_extensions() {
//         let supported_exts = vec![
//             Path::new("uwu.tar.xz"),
//             Path::new("uwu.tar.zst"),
//             Path::new("uwu.tar.zstd"),
//             Path::new("uwu.tar.gz"),
//         ];
//         for someext in supported_exts {
//             assert_eq!(true, get_compression_type(someext).is_ok());
//         }
//     }
// }
