use clap::Parser;
use obs_service_cargo_vendor_rs::cli;
use obs_service_cargo_vendor_rs::utils;
use std::io;
use tracing::{debug, info};

const PREFIX: &str = ".obs-service-cargo-vendor";
const VENDOR_EXAMPLE: &str = "
Examples of how to modify your spec file to use vendored libraries can be found online:

https://en.opensuse.org/Packaging_Rust_Software#Creating_the_Package

WARNING: To avoid cargo install rebuilding the binary in the install stage
         all environment variables must be the same as in the build stage.
";

// Create custom error type for processing

fn main() -> Result<(), io::Error> {
    tracing_subscriber::fmt::init();

    info!("🎢 Starting OBS Service Cargo Vendor.");
    let args = cli::Opts::parse();
    debug!(?args.srcdir, "SrcDir");
    let tmpdir = tempfile::Builder::new()
        .prefix(PREFIX)
        .rand_bytes(8)
        .tempdir()
        .expect("Failed to create temporary working directory.");
    let workdir = tmpdir.path();
    info!("Created temporary working directory: {:?}", workdir);

    // One is required over the other and there can't be both anyway.
    // NOTE: Because our struct `Opt` requires srctar or srcdir but not both, we put
    // some `unreachable!` macros because they cannot be reached after all.
    info!("Checking sources before vendor 🥡");
    if args.srcdir.is_some() {
        let src = match &args.srcdir {
            Some(val) => val,
            None => unreachable!(),
        };
        info!("Confirmed sources is a directory: {:?}", src.srcdir);
        utils::copy_dir_all(&src.srcdir, workdir)?;
        let prjdir = utils::get_manifest_file(workdir)?.to_owned();
        let prjdir = prjdir.parent().expect("Not parent directory");
        debug!("Guessed project root at {:?}", prjdir);
        src.vendor(&args, &src.srcdir)?;
    } else if args.srctar.is_some() {
        let src = match &args.srctar {
            Some(val) => val.to_owned(),
            None => unreachable!(),
        };
        info!(
            "Confirmed sources is a compressed tarball: {:?}",
            src.srctar
        );
        if src.srctar.exists() {
            src.decompress(workdir)?;
            let prjdir = utils::get_manifest_file(workdir)?.to_owned();
            let prjdir = prjdir.parent().expect("Not parent directory");
            debug!("Guessed project root at {:?}", prjdir);
            src.vendor(&args, prjdir)?;
        }
    } else {
        unreachable!()
    };

    info!("Vendor operation success! ❤️");
    info!("{}", VENDOR_EXAMPLE);

    // Remove temporary directory.
    tmpdir.close()?;
    info!("Successfully ran OBS Service Cargo Vendor 🥳");
    Ok(())
}
