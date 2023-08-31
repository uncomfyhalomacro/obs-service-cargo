use std::fs;
use std::io;
use std::path::Path;
use tar;

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
