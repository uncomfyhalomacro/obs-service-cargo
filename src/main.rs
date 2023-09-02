use clap::Parser;
use obs_service_cargo_vendor_rs::cli;
use obs_service_cargo_vendor_rs::utils;
use std::io;

const PREFIX: &str = ".obs-service-cargo-vendor";
// Create custom error type for processing

fn main() -> Result<(), io::Error> {
    let args = cli::Opts::parse();
    let tmpdir = tempfile::Builder::new()
        .prefix(PREFIX)
        .rand_bytes(8)
        .tempdir()
        .expect("Failed to create temporary working directory.");
    let workdir = tmpdir.path();
    // One is required over the other and there can't be both anyway.
    // NOTE: Because our struct `Opt` requires srctar or srcdir but not both, we put
    // some `unreachable!` macros because they cannot be reached after all.
    if args.srcdir.is_some() {
        let src = match &args.srcdir {
            Some(val) => val,
            None => unreachable!(),
        };
        src.vendor(&args, &src.srcdir)?;
    } else if args.srctar.is_some() {
        let src = match &args.srctar {
            Some(val) => val.to_owned(),
            None => unreachable!(),
        };
        if src.srctar.exists() {
            src.decompress(workdir)?;
            let prjdir = utils::get_manifest_file(workdir)?.to_owned();
            let prjdir = prjdir.parent().expect("Not parent directory");
            src.vendor(&args, prjdir)?;
        }
    } else {
        unreachable!()
    };

    // Remove temporary directory.
    tmpdir.close()?;
    Ok(())
}
