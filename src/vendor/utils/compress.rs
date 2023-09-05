use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::Path;
use tar;

#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

pub fn targz(
    topdir: &str,
    outdir: impl AsRef<Path>,
    srcpath: impl AsRef<Path>,
    additional_files: &[&str],
) -> Result<(), io::Error> {
    use flate2::write::GzEncoder;
    use flate2::Compression;

    let src = srcpath.as_ref().to_path_buf();
    let outtar = fs::File::create(outdir.as_ref())?;
    let enc = GzEncoder::new(outtar, Compression::default());
    let mut tar = tar::Builder::new(enc);
    if !additional_files.is_empty() {
        info!("Adding additional files!");
        for f in additional_files {
            let pathto = &src.join(f);
            info!(?pathto);
            let exists = pathto.exists();
            if exists {
                info!("Path to file or directory exists!");
                if pathto.is_file() {
                    info!("Path to is file!");
                    let basedir = pathto.file_name().unwrap_or(OsStr::new(f));
                    let mut addf = fs::File::open(pathto)?;
                    tar.append_file(basedir, &mut addf)?;
                    debug!("Added {} to archive", f);
                } else if pathto.is_dir() {
                    tar.append_dir_all("", pathto)?;
                    debug!("Added {} to archive", f);
                } else {
                    debug!("Is this the correct path to file? 🤔");
                };
            };
        }
    };
    tar.append_dir_all(topdir, &src)?;
    tar.finish()?;
    info!(
        "Successfully created Gz compressed archive for {}",
        src.to_string_lossy()
    );
    Ok(())
}

pub fn tarzst(
    topdir: &str,
    outdir: impl AsRef<Path>,
    srcpath: impl AsRef<Path>,
    additional_files: &[&str],
) -> Result<(), io::Error> {
    use zstd::Encoder;
    use zstd::DEFAULT_COMPRESSION_LEVEL;
    let src = srcpath.as_ref().to_path_buf();
    let outtar = fs::File::create(outdir.as_ref())?;
    let enc = Encoder::new(outtar, DEFAULT_COMPRESSION_LEVEL)?.auto_finish();
    let mut tar = tar::Builder::new(enc);
    if !additional_files.is_empty() {
        info!("Adding additional files!");
        for f in additional_files {
            let pathto = &src.join(f);
            info!(?pathto);
            let exists = pathto.exists();
            if exists {
                info!("Path to file or directory exists!");
                if pathto.is_file() {
                    info!("Path to is file!");
                    let basedir = pathto.file_name().unwrap_or(OsStr::new(f));
                    let mut addf = fs::File::open(pathto)?;
                    tar.append_file(basedir, &mut addf)?;
                    debug!("Added {} to archive", f);
                } else if pathto.is_dir() {
                    tar.append_dir_all("", pathto)?;
                    debug!("Added {} to archive", f);
                } else {
                    warn!("Is this the correct path to file? 🤔");
                };
            };
        }
    };
    tar.append_dir_all(topdir, &src)?;
    tar.finish()?;
    info!(
        "Successfully created Zstd compressed archive for {}",
        src.to_string_lossy()
    );
    Ok(())
}

pub fn tarxz(
    topdir: &str,
    outdir: impl AsRef<Path>,
    srcpath: impl AsRef<Path>,
    additional_files: &[&str],
) -> Result<(), io::Error> {
    use xz2::write::XzEncoder;
    let src = srcpath.as_ref().to_path_buf();
    let outtar = fs::File::create(outdir.as_ref())?;
    let enc = XzEncoder::new(outtar, 6);
    let mut tar = tar::Builder::new(enc);
    if !additional_files.is_empty() {
        info!("Adding additional files!");
        for f in additional_files {
            let pathto = &src.join(f);
            info!(?pathto);
            let exists = pathto.exists();
            if exists {
                info!("Path to file or directory exists!");
                if pathto.is_file() {
                    info!("Path to is file!");
                    let basedir = pathto.file_name().unwrap_or(OsStr::new(f));
                    let mut addf = fs::File::open(pathto)?;
                    tar.append_file(basedir, &mut addf)?;
                    debug!("Added {} to archive", f);
                } else if pathto.is_dir() {
                    tar.append_dir_all("", pathto)?;
                    debug!("Added {} to archive", f);
                } else {
                    warn!("Is this the correct path to file? 🤔");
                };
            };
        }
    };
    tar.append_dir_all(topdir, &src)?;
    tar.finish()?;
    info!(
        "Successfully created Xz compressed archive for {}",
        src.to_string_lossy()
    );
    Ok(())
}
