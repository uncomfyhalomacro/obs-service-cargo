use std::fs;
use std::io;
use std::io::Seek;
use std::path::Path;
use tar;

pub fn targz(outdir: impl AsRef<Path>, srcpath: impl AsRef<Path>) -> Result<(), io::Error> {
    use flate2::bufread::GzDecoder;
    let mut src = io::BufReader::new(fs::File::open(srcpath.as_ref())?);
    src.seek(io::SeekFrom::Start(0))?;
    let enc = GzDecoder::new(src);
    let mut ar = tar::Archive::new(enc);
    ar.unpack(outdir.as_ref())?;
    Ok(())
}

pub fn tarzst(outdir: impl AsRef<Path>, srcpath: impl AsRef<Path>) -> Result<(), io::Error> {
    use zstd::Decoder;
    let mut src = io::BufReader::new(fs::File::open(srcpath.as_ref())?);
    src.seek(io::SeekFrom::Start(0))?;
    let enc = Decoder::new(src)?;
    let mut ar = tar::Archive::new(enc);
    ar.unpack(outdir.as_ref())?;
    Ok(())
}

pub fn tarxz(outdir: impl AsRef<Path>, srcpath: impl AsRef<Path>) -> Result<(), io::Error> {
    use xz2::read::XzDecoder;
    let mut src = io::BufReader::new(fs::File::open(srcpath.as_ref())?);
    src.seek(io::SeekFrom::Start(0))?;
    let enc = XzDecoder::new(src);
    let mut ar = tar::Archive::new(enc);
    ar.unpack(outdir.as_ref())?;
    Ok(())
}
