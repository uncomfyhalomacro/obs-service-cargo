pub mod compress;
pub mod decompress;

use crate::cli::{Compression, Opts};
use crate::consts::{GZ_EXTS, GZ_MIME, SUPPORTED_MIME_TYPES, XZ_EXTS, XZ_MIME, ZST_EXTS, ZST_MIME};

use infer;
use std::error::Error;
use std::fmt;
use std::fmt::Debug;
use std::fs;
use std::io;
use std::os::unix::prelude::OsStrExt;
use std::path::{Path, PathBuf};
use std::process;

#[allow(unused_imports)]
use tracing::{debug, error, info, warn, Level};

pub fn get_manifest_file(srcdir: impl AsRef<Path>) -> Result<PathBuf, io::Error> {
    let target_file = "Cargo.toml";
    let dcanonical = srcdir
        .as_ref()
        .canonicalize()
        .expect("Canonical")
        .to_owned();
    for entry in std::fs::read_dir(&dcanonical)? {
        let mut dir: PathBuf = entry?.path().to_owned();
        dir.push(target_file);
        info!(?dir);
        let filename = unsafe {
            std::str::from_utf8_unchecked(dir.file_name().expect("Has base  name").as_bytes())
        };
        info!(filename);
        let isequal = filename == target_file;
        info!(isequal);
        info!(target_file);
        if filename == target_file {
            info!("Filename found?");
            return Ok(dir);
        }
    }
    debug!("SHOULD EXIST");
    Err(io::Error::new(
        io::ErrorKind::Other,
        "Not able to determine project root",
    ))
}

pub fn vendor(
    opts: impl AsRef<Opts>,
    prjdir: impl AsRef<Path>,
    vendorname: Option<&str>,
) -> Result<(), io::Error> {
    let mut prjdir = prjdir.as_ref().to_path_buf();
    info!(?prjdir);
    // Hack. This is to use the `current_dir` parameter of `std::process`.
    let mut manifest_path = prjdir.clone();
    manifest_path.push("Cargo.toml");
    info!(?manifest_path);
    let update = &opts.as_ref().update;
    let mut outdir = opts.as_ref().outdir.to_owned();
    let fullfilename = vendorname.unwrap_or("vendor");
    let mut cargo_config = String::new();
    if fullfilename == "vendor" {
        cargo_config.push_str("cargo_config");
    } else {
        let withprefix = format!("{}_cargo_config", fullfilename);
        cargo_config.push_str(&withprefix);
    };

    if *update {
        info!("Updating dependencies before vendor");
        let cargo_update = process::Command::new("cargo")
            .arg("update")
            .arg("-vv")
            .args(["--manifest-path", unsafe {
                std::str::from_utf8_unchecked(manifest_path.as_os_str().as_bytes())
            }])
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
        .args(["--manifest-path", unsafe {
            std::str::from_utf8_unchecked(manifest_path.as_os_str().as_bytes())
        }])
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
            let fullfilename_with_ext = format!("{}.tar.gz", fullfilename);
            outdir.push(&fullfilename_with_ext);
            if outdir.exists() {
                warn!(
                    ?outdir,
                    "Compressed tarball for vendor exists. Please manually check sources 🔦"
                );
            }
            debug!("Compressed to {}", outdir.to_string_lossy());
            compress::targz("vendor", outdir, &prjdir)?
        }
        Compression::Xz => {
            let fullfilename_with_ext = format!("{}.tar.xz", fullfilename);
            outdir.push(&fullfilename_with_ext);
            if outdir.exists() {
                warn!(
                    ?outdir,
                    "Compressed tarball for vendor exists. Please manually check sources 🔦"
                );
            }
            debug!("Compressed to {}", outdir.to_string_lossy());
            compress::tarxz("vendor", outdir, &prjdir)?
        }
        Compression::Zst => {
            let fullfilename_with_ext = format!("{}.tar.zst", fullfilename);
            outdir.push(&fullfilename_with_ext);
            if outdir.exists() {
                warn!(
                    ?outdir,
                    "Compressed tarball for vendor exists. Please manually check sources 🔦"
                );
            }
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
        let info = infer::get_from_path(file).expect("File is known");
        let extension = match file.extension() {
            Some(ext) => unsafe { std::str::from_utf8_unchecked(ext.as_bytes()) },
            None => "unknown extension",
        };
        let mimetype = match info {
            Some(ext) => ext.mime_type(),
            None => "unknown mime type",
        };
        if !SUPPORTED_MIME_TYPES.contains(&mimetype) {
            error!(?mimetype);
            Err(UnsupportedExtError {
                ext: Some(mimetype.to_string()),
            })
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
        let err = Err(UnsupportedExtError {
            ext: Some("Directory".to_string()),
        });
        error!(?err);
        err
    }
}

pub fn cargotomls(opts: impl AsRef<Opts>, workdir: impl AsRef<Path>) -> Result<(), io::Error> {
    info!("Vendoring separate crate!");
    let tomls = opts.as_ref().cargotoml.to_owned();
    info!(?tomls);
    for crateprj in tomls.iter() {
        let mut lrj = workdir.as_ref().to_owned();
        let exists = lrj.exists();
        debug!(exists);
        lrj.push(crateprj);
        lrj.pop();
        debug!(?crateprj);
        let exists = lrj.exists();
        debug!(?lrj);
        debug!(exists);
        let prjdir = get_manifest_file(&lrj)?
            .parent()
            .expect("Has parent")
            .to_owned();
        debug!(?prjdir);
        if let Some(has_basename) = lrj.file_name() {
            let prefix = unsafe { std::str::from_utf8_unchecked(has_basename.as_bytes()) };
            let vendorfilename = format!("{}-vendor", prefix);
            vendor(&opts, &lrj, Some(&vendorfilename))?;
        }
        info!(?crateprj, "Done vendoring subproject 🚀");
    }
    Ok(())
}
