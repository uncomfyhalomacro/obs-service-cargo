use clap::Parser;
use obs_service_cargo_vendor_rs::cli;
use obs_service_cargo_vendor_rs::consts::{PREFIX, VENDOR_EXAMPLE};
use obs_service_cargo_vendor_rs::utils;
use std::io;
use std::io::IsTerminal;
use terminfo::{capability as cap, Database};
use tracing_subscriber::EnvFilter;

#[allow(unused_imports)]
use tracing::{debug, error, info, warn, Level};

// Create custom error type for processing

fn main() -> Result<(), io::Error> {
    let args = cli::Opts::parse();
    let terminfodb = Database::from_env().expect("Loaded environment");
    let is_termcolorsupported = terminfodb.get::<cap::MaxColors>().is_some();
    let to_color = match std::io::stdout().is_terminal() {
        true => {
            let coloroption = &args.color;
            match coloroption {
                clap::ColorChoice::Auto => is_termcolorsupported,
                clap::ColorChoice::Always => true,
                clap::ColorChoice::Never => false,
            }
        }
        false => false,
    };

    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .expect("Env filter set");

    tracing_subscriber::fmt()
        .with_level(true)
        .with_ansi(to_color)
        .with_file(true)
        .with_line_number(true)
        .with_env_filter(filter_layer)
        .with_level(true)
        .init();

    info!("🎢 Starting OBS Service Cargo Vendor.");
    debug!(?args.srcdir);
    debug!(?args.srctar);
    let tmpdir = tempfile::Builder::new()
        .prefix(PREFIX)
        .rand_bytes(8)
        .tempdir()
        .expect("Failed to create temporary working directory.");
    let workdir = tmpdir.path();
    debug!("Created temporary working directory: {:?}", workdir);

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
        let prjdir = prjdir.parent().expect("Has a parent directory");
        debug!("Guessed project root at {:?}", prjdir);
        src.vendor(&args, prjdir)?;
        src.cargotomls(&args, prjdir)?;
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
            let prjdir = prjdir.parent().expect("Has a parent directory");
            debug!("Guessed project root at {:?}", prjdir);
            src.vendor(&args, prjdir)?;
            src.cargotomls(&args, prjdir)?;
        }
    } else {
        unreachable!()
    };

    info!("Vendor operation success! ❤️");
    info!("\n{}", VENDOR_EXAMPLE);

    // Remove temporary directory.
    tmpdir.close()?;
    info!("Successfully ran OBS Service Cargo Vendor 🥳");
    Ok(())
}
