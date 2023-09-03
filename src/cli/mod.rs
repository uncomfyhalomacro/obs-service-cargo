use crate::utils::{self, compress, decompress, get_compression_type, UnsupportedExtError};
use clap::{Args, Parser, ValueEnum};
use std::fmt::{self, Display};
use std::path::{Path, PathBuf};
use std::process;
use std::{fs, io};
use tracing::{debug, error, info, warn, Level};

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
    #[arg(long, default_value = "auto", help = "Whether to color output or not")]
    pub color: Option<clap::ColorChoice>,
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
    pub fn get_compression(&self) -> Result<Compression, UnsupportedExtError> {
        get_compression_type(&self.srctar)
    }
    pub fn decompress(&self, outdir: impl AsRef<Path>) -> Result<(), io::Error> {
        match self.get_compression() {
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
        utils::vendor(opts, prjdir)
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
        utils::vendor(opts, prjdir)
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
