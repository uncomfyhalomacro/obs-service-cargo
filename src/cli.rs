use clap::{Parser, ValueEnum};
use std::path::PathBuf;

use crate::utils::get_compression_type;

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

pub enum SrcKind {
    SrcTar,
    SrcDir,
}

pub trait Src {
    fn get_srckind(&self) -> Option<SrcKind>;
    fn srctar_compression_type(&self) -> Compression;
}

impl Src for Opts {
    fn get_srckind(&self) -> Option<SrcKind> {
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

    fn srctar_compression_type(&self) -> Compression {
        match self.get_srckind() {
            Some(kind) => {
                assert_eq!(true, matches!(kind, SrcKind::SrcTar))
            }
            None => panic!("Could not determine source kind"),
        }
        let compression_type = self.srctar.as_deref().map(|s| get_compression_type(s));
        match compression_type {
            None => panic!("Cannot determine compression type"),
            Some(c) => match c {
                Ok(t) => t,
                Err(err) => {
                    eprintln!("{}", err);
                    panic!();
                }
            },
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
            match opt.get_srckind() {
                Some(kind) => {
                    assert_eq!(true, matches!(kind, SrcKind::SrcDir));
                }
                None => panic!("Not satisfied"),
            }
        }

        #[test]
        #[should_panic]
        fn srcdir_errors_on_getting_compression_type() {
            let cmds = vec!["--", "--srcdir", "test", "--outdir", "test"];
            let opt = Opts::parse_from(&cmds);
            opt.srctar_compression_type();
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
            assert_eq!(
                true,
                matches!(opt.srctar_compression_type(), Compression::Xz)
            );
        }

        #[test]
        fn is_srctar_zst() {
            let cmds = vec!["--", "--srctar", "test.tar.zst", "--outdir", "test"];
            let opt = Opts::parse_from(&cmds);
            assert_eq!(
                true,
                matches!(opt.srctar_compression_type(), Compression::Zst)
            );
        }
        #[test]
        fn is_srctar_gz() {
            let cmds = vec!["--", "--srctar", "test.tar.gz", "--outdir", "test"];
            let opt = Opts::parse_from(&cmds);
            assert_eq!(
                true,
                matches!(opt.srctar_compression_type(), Compression::Gz)
            );
        }
    }
}
