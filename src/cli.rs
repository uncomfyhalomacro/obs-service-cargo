use clap::Parser;
use std::path::PathBuf;

#[command(
    author,
    name = "cargo_vendor",
    version,
    about = "OBS Source Service to vendor all crates.io and dependencies for Rust project locally",
    after_long_help = "Bugs can be reported on GitHub: https://github.com/uncomfyhalomacro/obs-service-cargo_vendor-rs/issues",
    max_term_width = 98
)]
#[derive(Parser, Debug)]
pub struct Opts {
    #[arg(long, help = "Where to find unpacked sources")]
    pub srcdir: PathBuf,
    #[arg(long, help = "Where to find packed sources")]
    pub srctar: PathBuf,
    #[arg(long, help = "Where to put vendor.tar* and cargo_config")]
    pub outdir: PathBuf,
    #[arg(long, help = "What compression algorithm to use e.g. zst")]
    pub compression: String,
    #[arg(
        long,
        help = "Tag some files for multi-vendor and multi-cargo_config projects"
    )]
    pub tag: String,
    #[arg(long, help = "Other cargo manifest files to sync with during vendor")]
    pub cargotoml: Vec<PathBuf>,
    #[arg(long, help = "Update dependencies or not")]
    pub update: bool,
}
