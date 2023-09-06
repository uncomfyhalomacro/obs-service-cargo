use clap::Parser;
use obs_service_cargo::cli;
use obs_service_cargo::consts::{PREFIX, VENDOR_EXAMPLE};
use obs_service_cargo::vendor::utils;
use obs_service_cargo::vendor::utils::is_workspace;
use std::io;
use std::io::IsTerminal;
use std::path::PathBuf;
use terminfo::{capability as cap, Database};
use tracing_subscriber::EnvFilter;

#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn, Level};

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
    let workdir: PathBuf = tmpdir.path().into();
    debug!("Created temporary working directory: {:?}", workdir);

    info!("Checking sources before vendor 🥡");
    if let Some(src) = &args.srcdir {
        info!("Confirmed sources is a directory: {:?}", src.srcdir);
        utils::copy_dir_all(&src.srcdir, &workdir)?;
        debug!(?workdir);
        let mut prjdir = utils::get_project_root(&workdir)?;
        debug!("Guessed project root at {:?}", prjdir);
        prjdir.push("Cargo.toml");
        if prjdir.exists() {
            if let Ok(isworkspace) = is_workspace(&prjdir) {
                if isworkspace {
                    info!("Project uses workspace! 👀");
                } else {
                    info!("Project not a workspace. Please check manually! 🫂");
                };
            };
            prjdir.pop();
            src.vendor(&args, &prjdir)?
        } else {
            warn!("This project seems to have no manifest file. Not vendoring based on project root. Please check manually");
        };
        if !args.cargotoml.is_empty() {
            info!("Subcrates to vendor found!");
            src.cargotomls(&args, &prjdir)?;
        } else {
            info!("No subcrates to vendor!");
        };
    };
    if let Some(src) = &args.srctar {
        info!(
            "Confirmed sources is a compressed tarball: {:?}",
            src.srctar
        );
        if src.srctar.exists() {
            src.decompress(&workdir)?;
            debug!(?workdir);
            let mut prjdir = utils::get_project_root(&workdir)?;
            // let prjdir = prjdir.parent().expect("Has a parent directory");
            debug!("Guessed project root at {:?}", prjdir);
            prjdir.push("Cargo.toml");
            if prjdir.exists() {
                if let Ok(isworkspace) = is_workspace(&prjdir) {
                    if isworkspace {
                        info!("Project uses workspace! 👀");
                    } else {
                        info!("Project not a workspace. Please check manually! 🫂");
                    };
                };
                prjdir.pop();
                src.vendor(&args, &prjdir)?
            } else {
                warn!("This project seems to have no manifest file. Not vendoring based on project root. Please check manually");
            };
            src.vendor(&args, &prjdir)?;
            if !args.cargotoml.is_empty() {
                info!("Subcrates to vendor found!");
                src.cargotomls(&args, &prjdir)?;
            } else {
                info!("No subcrates to vendor!");
            };
        }
    };
    info!("Vendor operation success! ❤️");
    info!("\n{}", VENDOR_EXAMPLE);

    // Remove temporary directory.
    tmpdir.close()?;
    info!("Successfully ran OBS Service Cargo Vendor 🥳");
    Ok(())
}
