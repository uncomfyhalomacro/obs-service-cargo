// SPDX-License-Identifier: MPL-2.0

// Copyright (C) 2023  Soc Virnyl Estela

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use clap::Parser;
use glob::glob;
use obs_service_cargo::cli::{self, SrcDir, SrcTar};
use obs_service_cargo::consts::{PREFIX, VENDOR_EXAMPLE};
use obs_service_cargo::vendor::utils;

use std::io;
use std::io::IsTerminal;
use std::path::PathBuf;
use terminfo::{capability as cap, Database};
use tracing_subscriber::EnvFilter;

#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn, Level};

// Create custom error type for processing

enum Src {
    Tar(SrcTar),
    Dir(SrcDir),
}

fn main() -> Result<(), io::Error> {
    let args = cli::Opts::parse();
    let terminfodb = Database::from_env().expect("Loaded environment");
    let is_termcolorsupported = terminfodb.get::<cap::MaxColors>().is_some();
    let to_color = matches!(std::io::stdout().is_terminal(), true if {
        let coloroption = &args.color;
        match coloroption {
            clap::ColorChoice::Auto => is_termcolorsupported,
            clap::ColorChoice::Always => true,
            clap::ColorChoice::Never => false,
        }
    });

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
        .pretty()
        .init();

    info!("🎢 Starting OBS Service Cargo Vendor.");
    debug!(?args);
    debug!(?args.srcdir);
    debug!(?args.srctar);
    let tmpdir = tempfile::Builder::new()
        .prefix(PREFIX)
        .rand_bytes(8)
        .tempdir()
        .expect("Failed to create temporary working directory.");
    let workdir: PathBuf = tmpdir.path().into();
    debug!("Created temporary working directory: {:?}", workdir);

    let src_type = match (&args.srcdir, &args.srctar) {
        (Some(srcdir), None) => Src::Dir(srcdir.clone()),
        (None, Some(srctar)) => Src::Tar(srctar.clone()),
        (Some(_), Some(_)) => {
            error!("Use only srcdir OR srctar - specifiying both is ambiguous");
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Use only srcdir OR srctar",
            ));
        }
        (None, None) => {
            error!("Must provide srcdir OR srctar");
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Must provide srcdir OR srctar",
            ));
        }
    };

    info!("Checking sources before vendor 🥡");

    let workdir = match src_type {
        Src::Dir(src) => {
            info!("Confirmed sources is a directory: {:?}", src.srcdir);
            let basename = &src.srcdir.file_name().unwrap_or(src.srcdir.as_os_str());
            let newworkdir = &workdir.join(basename);
            debug!(?newworkdir);
            utils::copy_dir_all(&src.srcdir, newworkdir)?;
            // Update the work dir
            newworkdir.clone()
        }
        Src::Tar(src) => {
            info!(
                "Confirmed sources is a compressed tarball: {:?}",
                src.srctar
            );

            let glob_iter = match glob(&src.srctar.as_os_str().to_string_lossy()) {
                Ok(gi) => gi,
                Err(e) => {
                    error!(err = ?e, "Invalid srctar glob input");
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Invalid srctar glob input",
                    ));
                }
            };

            let mut globs = glob_iter.into_iter().collect::<Vec<_>>();

            let matched_entry = match globs.len() {
                0 => {
                    error!("No files matched srctar glob input");
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "No files matched srctar glob input",
                    ));
                }
                1 => globs.pop().unwrap(),
                _ => {
                    error!("Multiple files matched srctar glob input");
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Multiple files matched srctar glob input",
                    ));
                }
            };

            debug!(?matched_entry, "Globbed result");
            match matched_entry {
                // Balls.
                Ok(balls) => {
                    let newsrc = SrcTar { srctar: balls };
                    if newsrc.srctar.exists() {
                        newsrc.decompress(&workdir)?;
                        debug!(?newsrc.srctar);
                        debug!(?workdir);
                        // Leave the workdir as is.
                        workdir
                    } else {
                        error!(?newsrc, "Source does not exist based on path");
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "Source does not exist based on path",
                        ));
                    }
                }
                Err(e) => {
                    error!(?e, "Got glob error");
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, "Glob error"));
                }
            }
        }
    };

    debug!(?workdir);
    match utils::get_project_root(&workdir) {
        Ok(prjdir) => {
            debug!("Guessed project root at {:?}", prjdir);
            // Addressed limitations of get_project_root
            let pathtomanifest = prjdir.join("Cargo.toml");
            if pathtomanifest.exists() {
                if let Ok(isworkspace) = utils::is_workspace(&pathtomanifest) {
                    if isworkspace {
                        info!("Project uses workspace! 👀");
                        if utils::has_dependencies(&pathtomanifest).unwrap_or(false) {
                            info!("Workspace has global dependencies!");
                        } else {
                            info!(
                                "No global dependencies! May vendor dependencies of member crates"
                            );
                        };
                    } else {
                        info!("Project is not a workspace. Please check manually! 🫂");
                        if utils::has_dependencies(&pathtomanifest).unwrap_or(false) {
                            info!("Project has dependencies!");
                        } else {
                            info!("No deps, no need to vendor!");
                        };
                    };
                };

                utils::vendor(&args, &prjdir, None)?;
                if !args.cargotoml.is_empty() {
                    info!("Subcrates to vendor found!");
                    utils::cargotomls(&args, &prjdir)?;
                } else {
                    info!("No subcrates to vendor!");
                };
            } else {
                warn!("This is not a rust project");
                warn!("Use the start of the root of the project to your subcrate instead!");
                // fallback to workdir
                utils::cargotomls(&args, &workdir)?;
            }
        }
        Err(err) => return Err(err),
    };

    info!("Vendor operation success! ❤️");
    info!("\n{}", VENDOR_EXAMPLE);

    // Remove temporary directory.
    tmpdir.close()?;
    info!("Successfully ran OBS Service Cargo Vendor 🥳");
    Ok(())
}
