use std::fs;
use std::io;
use std::path::Path;
use tar;

pub fn targz(outdir: impl AsRef<Path>, srcpath: impl AsRef<Path>) -> Result<(), io::Error> {
    use flate2::write::GzDecoder;

    let src = fs::File::open(srcpath.as_ref().to_path_buf())?;
    let enc = GzDecoder::new(src);
    let mut ar = tar::Archive::new(enc);
    ar.unpack(&outdir.as_ref())?;
    Ok(())
}

pub fn tarzst(outdir: impl AsRef<Path>, srcpath: impl AsRef<Path>) -> Result<(), io::Error> {
    use zstd::stream::Decoder;
    let src = fs::File::open(srcpath.as_ref().to_path_buf())?;
    let enc = Decoder::new(src)?;
    let mut ar = tar::Archive::new(enc);
    ar.unpack(&outdir.as_ref())?;
    Ok(())
}

pub fn tarxz(outdir: impl AsRef<Path>, srcpath: impl AsRef<Path>) -> Result<(), io::Error> {
    use xz2::write::XzDecoder;
    let src = fs::File::open(srcpath.as_ref().to_path_buf())?;
    let enc = XzDecoder::new(src);
    let mut ar = tar::Archive::new(enc);
    ar.unpack(&outdir.as_ref())?;
    Ok(())
}
