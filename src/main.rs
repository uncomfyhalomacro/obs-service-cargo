use clap::Parser;
use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

use obs_service_cargo_vendor_rs::{
    cli::{Compression, Opts, Src, SrcKind},
    utils,
};
use tempfile;

fn main() -> ExitCode {
    match run_cargo_vendor() {
        Ok(ok) => ok,
        Err(err) => err,
    }
}

fn run_cargo_vendor() -> Result<ExitCode, ExitCode> {
    let args = Opts::parse();
    let exit_status = match &args.srckind() {
        Some(kind) => {
            if matches!(kind, SrcKind::SrcTar) {
                let srcpath = args
                    .srctar
                    .as_deref()
                    .expect("Source tar cannot be determined");
                let compression_type = match args.srctar_compression_type() {
                    Ok(t) => t,
                    Err(err) => {
                        eprintln!("{}", err);
                        return Err(ExitCode::FAILURE);
                    }
                };

                process_srctar(&srcpath, &compression_type);
            } else if matches!(kind, SrcKind::SrcDir) {
                let srcpath = args
                    .srcdir
                    .as_deref()
                    .expect("Source dir cannot be determined");
                process_srcdir(&srcpath);
            }
            Ok(ExitCode::SUCCESS)
        }
        None => Err(ExitCode::FAILURE),
    };

    exit_status
}

fn process_srctar(srctar: impl AsRef<Path>, compression_type: &Compression) {}

fn process_srcdir(srcdir: impl AsRef<Path>) {}
