use clap::Parser;
use std::path::{Path, PathBuf};

use obs_service_cargo_vendor_rs::{
    cli::{Compression, Opts, Src, SrcKind},
    utils,
};
use tempfile;

fn main() {
    let args = Opts::parse();
    match &args.get_srckind() {
        Some(kind) => {
            if matches!(kind, SrcKind::SrcTar) {
                let srcpath = args
                    .srctar
                    .as_deref()
                    .expect("Source tar cannot be determined");
                let compression_type = args.srctar_compression_type();
                process_srctar(&srcpath, &compression_type);
            } else if matches!(kind, SrcKind::SrcDir) {
                let srcpath = args
                    .srcdir
                    .as_deref()
                    .expect("Source dir cannot be determined");
                process_srcdir(&srcpath);
            }
        }
        None => panic!("Not satisfied"),
    }
}

fn process_srctar(srctar: impl AsRef<Path>, compression_type: &Compression) {}

fn process_srcdir(srcdir: impl AsRef<Path>) {}
