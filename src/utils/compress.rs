use std::fs;
use std::io;
use std::path::Path;
use tar;

/*
NOTE: See https://docs.rs/async-compression/latest/async_compression
I doubt we even need that but just putting the link there in case
TODO: Make this work for general stuff. Strcitly using it for vendor defeats its purpose
*/
pub fn targz(outdir: impl AsRef<Path>, srcpath: impl AsRef<Path>) -> Result<(), io::Error> {
    use flate2::write::GzEncoder;
    use flate2::Compression;

    let src = srcpath.as_ref().to_path_buf();
    let outtar = fs::File::create(outdir.as_ref())?;
    let enc = GzEncoder::new(outtar, Compression::default());
    let mut tar = tar::Builder::new(enc);
    tar.append_dir_all(".", &src)?;
    tar.finish()?;
    Ok(())
}

pub fn tarzst(outdir: impl AsRef<Path>, srcpath: impl AsRef<Path>) -> Result<(), io::Error> {
    use zstd::stream::Encoder;
    use zstd::DEFAULT_COMPRESSION_LEVEL;
    let src = srcpath.as_ref().to_path_buf();
    let outtar = fs::File::create(outdir.as_ref())?;
    let enc = Encoder::new(outtar, DEFAULT_COMPRESSION_LEVEL)?.auto_finish();
    let mut tar = tar::Builder::new(enc);
    tar.append_dir_all(".", &src)?;
    tar.finish()?;
    Ok(())
}

pub fn tarxz(outdir: impl AsRef<Path>, srcpath: impl AsRef<Path>) -> Result<(), io::Error> {
    use xz2::write::XzEncoder;

    let src = srcpath.as_ref().to_path_buf();
    let outtar = fs::File::create(outdir.as_ref())?;
    let enc = XzEncoder::new(outtar, 6);
    let mut tar = tar::Builder::new(enc);
    tar.append_dir_all(".", &src)?;
    tar.finish()?;
    Ok(())
}
