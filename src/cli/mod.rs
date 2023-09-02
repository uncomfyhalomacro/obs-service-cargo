use clap::{Args, Parser, ValueEnum};
use std::fmt::{self, Display};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;
use std::str::EscapeDebug;
use std::{fs, io};
use tracing::{debug, error, info, span, warn, Level};

use crate::utils::{compress, decompress, get_compression_type, UnsupportedExtError};

#[derive(Parser, Debug)]
#[command(
    author,
    name = "cargo_vendor",
    version,
    about = "OBS Source Service to vendor all crates.io and dependencies for Rust project locally",
    after_long_help = "Bugs can be reported on GitHub: https://github.com/uncomfyhalomacro/obs-service-cargo_vendor-rs/issues",
    max_term_width = 120
)]
pub struct Opts {
    #[clap(flatten)]
    pub srctar: Option<SrcTar>,
    #[clap(flatten)]
    pub srcdir: Option<SrcDir>,
    #[arg(
        long,
        value_enum,
        default_value_t,
        help = "What compression algorithm to use."
    )]
    pub compression: Compression,
    #[arg(
        long,
        help = "Tag some files for multi-vendor and multi-cargo_config projects"
    )]
    pub tag: Option<String>,
    #[arg(long, help = "Other cargo manifest files to sync with during vendor")]
    pub cargotoml: Vec<PathBuf>,
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set, help = "Update dependencies or not")]
    pub update: bool,
    #[arg(long, help = "Where to output vendor.tar* and cargo_config")]
    pub outdir: PathBuf,
}

impl AsRef<Opts> for Opts {
    #[inline]
    fn as_ref(&self) -> &Opts {
        self
    }
}

#[derive(Args, Debug, Clone)]
pub struct SrcTar {
    #[arg(long, help = "Where to find packed sources", conflicts_with = "srcdir")]
    pub srctar: PathBuf,
}

impl SrcTar {
    pub fn extension(&self) -> Result<Compression, UnsupportedExtError> {
        get_compression_type(&self.srctar)
    }

    pub fn decompress(&self, outdir: impl AsRef<Path>) -> Result<(), io::Error> {
        match self.extension() {
            Ok(comp) => match comp {
                Compression::Gz => decompress::targz(outdir.as_ref(), &self.srctar),
                Compression::Xz => decompress::tarxz(outdir.as_ref(), &self.srctar),
                Compression::Zst => decompress::tarzst(outdir.as_ref(), &self.srctar),
            },
            Err(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
        }
    }

    pub fn vendor(
        &self,
        opts: impl AsRef<Opts>,
        prjdir: impl AsRef<Path>,
    ) -> Result<(), io::Error> {
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
                compress::targz("vendor", outdir, &prjdir)?
            }
            Compression::Xz => {
                outdir.push("vendor.tar.xz");
                compress::tarxz("vendor", outdir, &prjdir)?
            }
            Compression::Zst => {
                outdir.push("vendor.tar.zst");
                compress::tarzst("vendor", outdir, &prjdir)?
            }
        };
        info!("Finished creating {} compressed tarball", compression);
        Ok(())
    }
}

#[derive(Args, Debug, Clone)]
pub struct SrcDir {
    #[arg(
        long,
        help = "Where to find unpacked sources",
        conflicts_with = "srctar"
    )]
    pub srcdir: PathBuf,
}

impl SrcDir {
    pub fn vendor(
        &self,
        opts: impl AsRef<Opts>,
        prjdir: impl AsRef<Path>,
    ) -> Result<(), io::Error> {
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
                compress::targz("vendor", outdir, &prjdir)?
            }
            Compression::Xz => {
                outdir.push("vendor.tar.xz");
                compress::tarxz("vendor", outdir, &prjdir)?
            }
            Compression::Zst => {
                outdir.push("vendor.tar.zst");
                compress::tarzst("vendor", outdir, &prjdir)?
            }
        };
        info!("Finished creating {} compressed tarball", compression);
        Ok(())
    }
}

#[derive(ValueEnum, Default, Debug, Clone)]
pub enum Compression {
    Gz,
    Xz,
    #[default]
    Zst,
}

impl Display for Compression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            Compression::Gz => "gz",
            Compression::Xz => "xz",
            Compression::Zst => "zst",
        };
        write!(f, "{}", msg)
    }
}
