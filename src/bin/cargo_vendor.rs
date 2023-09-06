// SPDX-License-Identifier: GPL-2.0

// Copyright (C) 2023  Soc Virnyl Estela

// This program is free software; you can redistribute it and/or
// modify it under the terms of the GNU General Public License
// as published by the Free Software Foundation; either version 2
// of the License, or (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program; if not, write to the Free Software
// Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA
// 02110-1301, USA.

use clap::Parser;
use obs_service_cargo::cli;
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

    info!("Checking sources before vendor 🥡");
    if let Some(src) = &args.srcdir {
        info!("Confirmed sources is a directory: {:?}", src.srcdir);
        let basename = &src.srcdir.file_name().unwrap_or(src.srcdir.as_os_str());
        let newworkdir = &workdir.join(basename);
        debug!(?newworkdir);
        utils::copy_dir_all(&src.srcdir, newworkdir)?;
        debug!(?workdir);
        // You will still use workdir just in case ;) so its behavior is the same as srctar
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
                                info!("No global dependencies! May vendor dependencies of member crates");
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

                    src.vendor(&args, &prjdir)?;
                    if !args.cargotoml.is_empty() {
                        info!("Subcrates to vendor found!");
                        src.cargotomls(&args, &prjdir)?;
                    } else {
                        info!("No subcrates to vendor!");
                    };
                } else {
                    warn!("This is not a rust project");
                    warn!("Use the start of the root of the project to your subcrate instead!");
                    // fallback to workdir
                    src.cargotomls(&args, &workdir)?;
                }
            }
            Err(err) => return Err(err),
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
                                    info!("No global dependencies! May vendor dependencies of member crates");
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

                        src.vendor(&args, &prjdir)?;
                        if !args.cargotoml.is_empty() {
                            info!("Subcrates to vendor found!");
                            src.cargotomls(&args, &prjdir)?;
                        } else {
                            info!("No subcrates to vendor!");
                        };
                    } else {
                        warn!("This is not a rust project");
                        warn!("Use the start of the root of the project to your subcrate instead!");
                        src.cargotomls(&args, &workdir)?;
                    }
                }
                Err(err) => return Err(err),
            };
        };
    };
    info!("Vendor operation success! ❤️");
    info!("\n{}", VENDOR_EXAMPLE);

    // Remove temporary directory.
    tmpdir.close()?;
    info!("Successfully ran OBS Service Cargo Vendor 🥳");
    Ok(())
}
