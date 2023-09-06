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

use std::fs;
use std::io;
use std::io::Seek;
use std::path::Path;
use tar;

#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

pub fn targz(outdir: impl AsRef<Path>, srcpath: impl AsRef<Path>) -> Result<(), io::Error> {
    use flate2::bufread::GzDecoder;
    let mut src = io::BufReader::new(fs::File::open(srcpath.as_ref())?);
    src.seek(io::SeekFrom::Start(0))?;
    let enc = GzDecoder::new(src);
    let mut ar = tar::Archive::new(enc);
    ar.unpack(outdir.as_ref())?;
    info!(
        "Successfully created Gz decompressed archive for {}",
        srcpath.as_ref().to_string_lossy()
    );
    Ok(())
}

pub fn tarzst(outdir: impl AsRef<Path>, srcpath: impl AsRef<Path>) -> Result<(), io::Error> {
    use zstd::Decoder;
    let mut src = io::BufReader::new(fs::File::open(srcpath.as_ref())?);
    src.seek(io::SeekFrom::Start(0))?;
    let enc = Decoder::new(src)?;
    let mut ar = tar::Archive::new(enc);
    ar.unpack(outdir.as_ref())?;
    info!(
        "Successfully created Zst decompressed archive for {}",
        srcpath.as_ref().to_string_lossy()
    );
    Ok(())
}

pub fn tarxz(outdir: impl AsRef<Path>, srcpath: impl AsRef<Path>) -> Result<(), io::Error> {
    use xz2::read::XzDecoder;
    let mut src = io::BufReader::new(fs::File::open(srcpath.as_ref())?);
    src.seek(io::SeekFrom::Start(0))?;
    let enc = XzDecoder::new(src);
    let mut ar = tar::Archive::new(enc);
    ar.unpack(outdir.as_ref())?;
    info!(
        "Successfully created Xz decompressed archive for {}",
        srcpath.as_ref().to_string_lossy()
    );
    Ok(())
}
