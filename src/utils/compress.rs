use std::fs;
use std::io;
use std::path::Path;
use tar;

// NOTE: See https://docs.rs/async-compression/latest/async_compression
// I doubt we even need that but just putting the link there in case
pub fn targz(outdir: impl AsRef<Path>, prjroot: impl AsRef<Path>) -> Result<(), io::Error> {
    use flate2::write::GzEncoder;
    use flate2::Compression;

    let mut vendorsrc = prjroot.as_ref().to_path_buf();
    vendorsrc.push("vendor/");
    let vendortar = fs::File::create(format!(
        "{}/vendor.tar.gz",
        &outdir.as_ref().to_str().unwrap()
    ))?;
    let enc = GzEncoder::new(vendortar, Compression::default());
    let mut tar = tar::Builder::new(enc);
    tar.append_dir_all("vendor/", &vendorsrc)?;
    tar.finish()?;
    Ok(())
}

pub fn tarzst(outdir: impl AsRef<Path>, prjroot: impl AsRef<Path>) -> Result<(), io::Error> {
    use zstd::stream::Encoder;
    use zstd::DEFAULT_COMPRESSION_LEVEL;
    let mut vendorsrc = prjroot.as_ref().to_path_buf();
    vendorsrc.push("vendor/");
    let vendortar = fs::File::create(format!(
        "{}/vendor.tar.zst",
        &outdir.as_ref().to_str().unwrap()
    ))?;
    let enc = Encoder::new(vendortar, DEFAULT_COMPRESSION_LEVEL)?.auto_finish();
    let mut tar = tar::Builder::new(enc);
    tar.append_dir_all("vendor/", &vendorsrc)?;
    tar.finish()?;
    Ok(())
}

pub fn tarxz(outdir: impl AsRef<Path>, prjroot: impl AsRef<Path>) -> Result<(), io::Error> {
    use xz2::write::XzEncoder;

    let mut vendorsrc = prjroot.as_ref().to_path_buf();
    vendorsrc.push("vendor/");
    let vendortar = fs::File::create(format!(
        "{}/vendor.tar.xz",
        &outdir.as_ref().to_str().unwrap()
    ))?;
    let enc = XzEncoder::new(vendortar, 6);
    let mut tar = tar::Builder::new(enc);
    tar.append_dir_all("vendor/", &vendorsrc)?;
    tar.finish()?;
    Ok(())
}
