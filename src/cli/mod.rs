use clap::{Args, Parser, ValueEnum};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;
use std::{fs, io};

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
    #[arg(long, default_value_t = true, help = "Update dependencies or not")]
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

    // NOTE: outdir is a TempDir
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
            println!("Updated dependencies before vendor");
            let cargo_update = process::Command::new("cargo")
                .arg("update")
                .arg("-vv")
                .current_dir(&prjdir)
                .output()
                .expect("Failed to run cargo update.");
            if !cargo_update.status.success() {
                io::stderr()
                    .write_all(&cargo_update.stderr)
                    .expect("Failed to write stderr.");
                panic!("Failed to run cargo update.")
            } else {
                io::stdout()
                    .write_all(&cargo_update.stdout)
                    .expect("Failed to write stdout.");
            }
        } else {
            println!("Disabled update of dependencies. You may reenable it for security updates.")
        };

        let cargo_vendor = process::Command::new("cargo")
            .arg("vendor")
            .arg("-vv")
            .current_dir(&prjdir)
            .output()
            .expect("Failed to run cargo vendor");

        if !cargo_vendor.status.success() {
            io::stderr()
                .write_all(&cargo_vendor.stderr)
                .expect("Failed to write error message");
            panic!("Failed to run cargo vendor")
        } else {
            io::stdout()
                .write_all(&cargo_vendor.stdout)
                .expect("Failed to write stdout.");
            fs::write(cargo_config, &cargo_vendor.stdout)?;
        };
        println!("Proceeding to create compressed archive of vendored deps...");
        prjdir.push("vendor/");
        let compression: &Compression = &opts.as_ref().compression;
        match compression {
            Compression::Gz => {
                outdir.push("vendor.tar.gz");
                compress::targz(outdir, &prjdir)?
            }
            Compression::Xz => {
                outdir.push("vendor.tar.xz");
                compress::tarxz(outdir, &prjdir)?
            }
            Compression::Zst => {
                outdir.push("vendor.tar.zst");
                compress::tarzst(outdir, &prjdir)?
            }
        };
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
            println!("Updated dependencies before vendor");
            let cargo_update = process::Command::new("cargo")
                .arg("update")
                .arg("-vv")
                .current_dir(&prjdir)
                .output()
                .expect("Failed to run cargo update.");
            if !cargo_update.status.success() {
                io::stderr()
                    .write_all(&cargo_update.stderr)
                    .expect("Failed to write stderr.");
                panic!("Failed to run cargo update.")
            } else {
                io::stdout()
                    .write_all(&cargo_update.stdout)
                    .expect("Failed to write stdout.");
            }
        } else {
            println!("Disabled update of dependencies. You may reenable it for security updates.")
        };

        let cargo_vendor = process::Command::new("cargo")
            .arg("vendor")
            .arg("-vv")
            .current_dir(&prjdir)
            .output()
            .expect("Failed to run cargo vendor");

        if !cargo_vendor.status.success() {
            io::stderr()
                .write_all(&cargo_vendor.stderr)
                .expect("Failed to write error message");
            panic!("Failed to run cargo vendor")
        } else {
            io::stdout()
                .write_all(&cargo_vendor.stdout)
                .expect("Failed to write stdout.");
            fs::write(cargo_config, &cargo_vendor.stdout)?;
        };
        println!("Proceeding to create compressed archive of vendored deps...");
        prjdir.push("vendor/");
        let compression: &Compression = &opts.as_ref().compression;
        match compression {
            Compression::Gz => {
                outdir.push("vendor.tar.gz");
                compress::targz(outdir, &prjdir)?
            }
            Compression::Xz => {
                outdir.push("vendor.tar.xz");
                compress::tarxz(outdir, &prjdir)?
            }
            Compression::Zst => {
                outdir.push("vendor.tar.zst");
                compress::tarzst(outdir, &prjdir)?
            }
        };
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
