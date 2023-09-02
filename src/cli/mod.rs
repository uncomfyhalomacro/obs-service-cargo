use clap::{Args, Parser, ValueEnum};
use std::error::Error;
use std::fmt::{Debug, Display};
use std::path::PathBuf;
use std::{fmt, io};

use crate::utils::{decompress, get_compression_type, UnsupportedExtError};

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
    #[arg(long, default_value_t, help = "Update dependencies or not")]
    pub update: bool,
    #[arg(long, help = "Where to output vendor.tar* and cargo_config")]
    pub outdir: PathBuf,
}

#[derive(Args, Debug)]
pub struct SrcTar {
    #[arg(long, help = "Where to find packed sources", conflicts_with = "srcdir")]
    pub srctar: PathBuf,
}

impl SrcTar {
    fn extension(&self) -> Result<Compression, UnsupportedExtError> {
        get_compression_type(&self.srctar)
    }

    fn decompress(&self, outdir: PathBuf) -> Result<(), io::Error> {
        match self.extension() {
            Ok(comp) => match comp {
                Compression::Gz => decompress::targz(outdir, &self.srctar),
                Compression::Xz => decompress::tarxz(outdir, &self.srctar),
                Compression::Zst => decompress::tarzst(outdir, &self.srctar),
            },
            Err(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
        }
    }
    
    fn vendor(&self) -> Result<(), io::Error> {
        todo!()
    }
}

#[derive(Args, Debug)]
pub struct SrcDir {
    #[arg(
        long,
        help = "Where to find unpacked sources",
        conflicts_with = "srctar"
    )]
    pub srcdir: PathBuf,
}

impl SrcDir {
    fn vendor(&self) -> Result<(), io::Error> {
        todo!()
    }
}

#[derive(ValueEnum, Default, Debug, Clone)]
pub enum Compression {
    Gz,
    Xz,
    #[default]
    Zst,
}
