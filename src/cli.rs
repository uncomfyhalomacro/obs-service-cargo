use crate::utils::{get_compression_type, UnsupportedExtError};
use clap::{Parser, ValueEnum};
use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display};
use std::{error, path::PathBuf};

#[derive(ValueEnum, Default, Debug, Clone)]
pub enum Compression {
    Gz,
    Xz,
    #[default]
    Zst,
}

#[derive(Parser, Debug)]
#[command(
    author,
    name = "cargo_vendor",
    version,
    about = "OBS Source Service to vendor all crates.io and dependencies for Rust project locally",
    after_long_help = "Bugs can be reported on GitHub: https://github.com/uncomfyhalomacro/obs-service-cargo_vendor-rs/issues",
    max_term_width = 98
)]
pub struct Opts {
    #[arg(
        long,
        help = "Where to find unpacked sources",
        conflicts_with = "srctar"
    )]
    pub srcdir: Option<PathBuf>,

    #[arg(long, help = "Where to find packed sources")]
    pub srctar: Option<PathBuf>,

    #[arg(long, help = "Where to output vendor.tar* and cargo_config")]
    pub outdir: PathBuf,
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
}

#[derive(Debug)]
pub enum SrcKind {
    SrcTar,
    SrcDir,
}

#[derive(Debug)]
struct SrcKindError {
    src: Option<SrcKind>,
}

impl fmt::Display for SrcKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SrcKind::SrcTar => write!(f, "SrcKind::SrcTar"),
            SrcKind::SrcDir => write!(f, "SrcKind::SrcDir"),
        }
    }
}

impl fmt::Display for SrcKindError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let src = &self.src;
        let msg = match src {
            Some(kind) => {
                if matches!(kind, SrcKind::SrcTar) {
                    format!("Expected `SrcKind::SrcDir`, got `{}`", kind)
                } else {
                    format!("Expected `SrcKind::SrcTar`, got `{}`", kind)
                }
            }
            None => "Could not determine src kind".to_string(),
        };

        write!(f, "{}", &msg)
    }
}

impl Error for SrcKindError {}

pub trait Src {
    fn srckind(&self) -> Option<SrcKind>;
    fn srctar_compression_type(&self) -> Result<Compression, Box<dyn error::Error>>;
}

impl Src for Opts {
    fn srckind(&self) -> Option<SrcKind> {
        if self.srcdir.is_some() {
            self.srcdir
                .as_deref()
                .map(|_| -> SrcKind { SrcKind::SrcDir })
        } else if self.srctar.is_some() {
            self.srctar
                .as_deref()
                .map(|_| -> SrcKind { SrcKind::SrcTar })
        } else {
            None
        }
    }

    fn srctar_compression_type(&self) -> Result<Compression, Box<dyn error::Error>> {
        match self.srckind() {
            Some(kind) => {
                if matches!(kind, SrcKind::SrcTar) {
                    let compression_type = self.srctar.as_deref().map(|s| get_compression_type(s));
                    match compression_type {
                        None => panic!(),
                        Some(c) => match c {
                            Ok(t) => Ok(t),
                            Err(err) => Err(Box::new(err)),
                        },
                    }
                } else {
                    Err(Box::new(SrcKindError { src: Some(kind) }))
                }
            }
            None => Err(Box::new(SrcKindError { src: None })),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    mod srcdir {
        use super::*;

        #[test]
        fn is_using_srcdir() {
            // Always remember to add `--` as the first arg.
            let cmds = vec!["--", "--srcdir", "test", "--outdir", "test"];
            let opt = Opts::parse_from(&cmds);
            opt.srckind()
                .map(|f| assert_eq!(true, matches!(f, SrcKind::SrcDir)));
        }

        #[test]
        fn srcdir_errors_on_getting_compression_type() {
            let cmds = vec!["--", "--srcdir", "test", "--outdir", "test"];
            let opt = Opts::parse_from(&cmds);
            assert_eq!(true, opt.srctar_compression_type().is_err());
        }
    }

    mod srctar {
        use super::*;

        #[test]
        fn is_using_srctar() {
            // Always remember to add `--` as the first arg.
            let cmds = vec!["--", "--srctar", "test", "--outdir", "test"];
            let opt = Opts::parse_from(&cmds);
        }

        #[test]
        fn is_srctar_xz() {
            let cmds = vec!["--", "--srctar", "test.tar.xz", "--outdir", "test"];
            let opt = Opts::parse_from(&cmds);
            match opt.srctar_compression_type() {
                Ok(c) => assert_eq!(true, matches!(c, Compression::Xz)),
                Err(err) => {
                    eprintln!("Expected xz or lzma compressed file, got `{}`", err);
                    panic!("Not an xz compressed tar file")
                }
            }
        }

        #[test]
        fn is_srctar_zst() {
            let cmds = vec!["--", "--srctar", "test.tar.zst", "--outdir", "test"];
            let opt = Opts::parse_from(&cmds);
            match opt.srctar_compression_type() {
                Ok(c) => assert_eq!(true, matches!(c, Compression::Zst)),
                Err(err) => {
                    eprintln!("Expected zstd compressed file, got `{}`", err);
                    panic!("Not a zst compressed tar file")
                }
            }
        }
        #[test]
        fn is_srctar_gz() {
            let cmds = vec!["--", "--srctar", "test.tar.gz", "--outdir", "test"];
            let opt = Opts::parse_from(&cmds);
            match opt.srctar_compression_type() {
                Ok(c) => assert_eq!(true, matches!(c, Compression::Gz)),
                Err(err) => {
                    eprintln!("Expected gz compressed file, got `{}`", err);
                    panic!("Not a gz compressed tar file")
                }
            }
        }
    }
}
