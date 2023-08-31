use clap::Parser;
use obs_service_cargo_vendor_rs::{
    cli::{Compression, Opts, Src, SrcKind},
    utils::{self, cargo_vendor},
};
use std::io::prelude::*;
use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};
use tar;
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
                    .expect("Source tar cannot be determined")
                    .to_path_buf()
                    .canonicalize()
                    .unwrap();
                let compression_type = match args.srctar_compression_type() {
                    Ok(t) => t,
                    Err(err) => {
                        eprintln!("{}", err);
                        return Err(ExitCode::FAILURE);
                    }
                };

                let vendor_compression_type = args.compression;
                let update: bool = args.update;

                process_srctar(&srcpath, compression_type, vendor_compression_type, update);
            } else if matches!(kind, SrcKind::SrcDir) {
                let srcpath = args
                    .srcdir
                    .as_deref()
                    .expect("Source dir cannot be determined")
                    .to_path_buf()
                    .canonicalize()
                    .unwrap()
                    .to_path_buf();

                let vendor_compression_type = args.compression;
                let update: bool = args.update;
                let tag = args.tag;
                let outdir = args.outdir;

                // I wonder if this can be just a trait method? 🤔
                process_srcdir(&srcpath, vendor_compression_type, update, tag, outdir);
            }
            Ok(ExitCode::SUCCESS)
        }
        None => Err(ExitCode::FAILURE),
    };

    exit_status
}

fn process_srctar(
    srctar: impl AsRef<Path>,
    srctar_compression_type: Compression,
    vendor_compression_type: Compression,
    update: bool,
) {
}

fn process_srcdir(
    srcdir: impl AsRef<Path>,
    vendor_compression_type: Compression,
    update: bool,
    tag: Option<String>,
    outdir: impl AsRef<Path>,
) {
    match tempfile::Builder::new()
        .prefix(".obs-service-cargo-vendor")
        .rand_bytes(8)
        .tempdir()
    {
        Ok(dir) => {
            let basename = &srcdir.as_ref().file_name().expect("No basename");
            let dir = dir.path().join(&basename);
            utils::copy_dir_all(&srcdir, &dir).expect("Cannot copy");
            let mut prjroot = utils::get_manifest_file(&dir)
                .expect("Something went wrong")
                .parent()
                .expect("What is parent path?")
                .to_owned();
            utils::cargo_vendor(&prjroot).expect("Error. Cannot vendor");
            let vendor_tar_path = format!("{}/vendor.tar", &outdir.as_ref().to_str().unwrap());
            println!("{}", &vendor_tar_path);
            let vendor_tar_path = fs::File::create(&vendor_tar_path).unwrap();
            let mut ar = tar::Builder::new(vendor_tar_path);
            prjroot.push("vendor");
            ar.append_dir_all("vendor/", &prjroot).unwrap();
            ar.finish().expect("Something wrong");
        }
        Err(err) => {
            eprintln! {"{}", err};
            panic!("Something went wrong!");
        }
    };
}
