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

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use tar;

#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

pub fn tar_builder<T>(
    topdir: &str,
    srcpath: impl AsRef<Path>,
    additional_files: &[&str],
    builder: &mut tar::Builder<T>,
) -> Result<(), io::Error>
where
    T: Write,
{
    if !additional_files.is_empty() {
        info!("Adding additional files!");
        for f in additional_files {
            let pathto = &srcpath.as_ref().join(f);
            info!(?pathto);
            let exists = pathto.exists();
            if exists {
                warn!(?pathto, "Path to file or directory exists!");
                if pathto.is_file() {
                    debug!(?pathto, "Path to is file!");
                    let basedir = pathto.file_name().unwrap_or(OsStr::new(f));
                    let mut addf = fs::File::open(pathto)?;
                    builder.append_file(basedir, &mut addf)?;
                    debug!("Added {} to archive", f);
                } else if pathto.is_dir() {
                    builder.append_dir_all("", pathto)?;
                    debug!("Added {} to archive", f);
                } else {
                    warn!(?pathto, "Is this the correct path to file? 🤔");
                };
            };
        }
    };
    builder.append_dir_all(topdir, &srcpath)?;
    builder.finish()?;
    info!(
        "Successfully created Xz compressed archive for {}",
        srcpath.as_ref().to_string_lossy()
    );
    Ok(())
}

pub fn targz(
    topdir: &str,
    outdir: impl AsRef<Path>,
    srcpath: impl AsRef<Path>,
    additional_files: &[&str],
) -> Result<(), io::Error> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    let outtar = fs::File::create(outdir.as_ref())?;
    let encoder = GzEncoder::new(outtar, Compression::default());
    let mut builder = tar::Builder::new(encoder);
    tar_builder(topdir, srcpath, additional_files, &mut builder)
}

pub fn tarzst(
    topdir: &str,
    outdir: impl AsRef<Path>,
    srcpath: impl AsRef<Path>,
    additional_files: &[&str],
) -> Result<(), io::Error> {
    use zstd::Encoder;
    let outtar = fs::File::create(outdir.as_ref())?;
    let mut enc_builder = Encoder::new(outtar, 19)?;
    enc_builder.include_checksum(true)?;
    let threads: u32 = std::thread::available_parallelism()?.get() as u32;
    enc_builder.multithread(threads)?;
    let encoder = enc_builder.auto_finish();
    let mut builder = tar::Builder::new(encoder);
    tar_builder(topdir, srcpath, additional_files, &mut builder)
}

pub fn tarxz(
    topdir: &str,
    outdir: impl AsRef<Path>,
    srcpath: impl AsRef<Path>,
    additional_files: &[&str],
) -> Result<(), io::Error> {
    use xz2::stream::Check::Sha256;
    use xz2::stream::MtStreamBuilder;
    use xz2::write::XzEncoder;
    let outtar = fs::File::create(outdir.as_ref())?;
    let threads: u32 = std::thread::available_parallelism()?.get() as u32;
    let enc_builder = MtStreamBuilder::new()
        .preset(6)
        .threads(threads)
        .check(Sha256)
        .encoder()?;
    let encoder = XzEncoder::new_stream(outtar, enc_builder);
    let mut builder = tar::Builder::new(encoder);
    tar_builder(topdir, srcpath, additional_files, &mut builder)
}
