pub const PREFIX: &str = ".obs-service-cargo-vendor";
pub const VENDOR_EXAMPLE: &str =
    "Examples of how to modify your spec file to use vendored libraries can be found online:

https://en.opensuse.org/Packaging_Rust_Software#Creating_the_Package

WARNING: To avoid cargo install rebuilding the binary in the install stage
         all environment variables must be the same as in the build stage.
";

pub const XZ_EXTS: &[&str] = &["xz"];
pub const ZST_EXTS: &[&str] = &["zstd", "zst"];
pub const GZ_EXTS: &[&str] = &["gz", "gzip"];
pub const XZ_MIME: &str = "application/x-xz";
pub const ZST_MIME: &str = "application/zstd";
pub const GZ_MIME: &str = "application/gzip";
pub const SUPPORTED_MIME_TYPES: &[&str] = &[XZ_MIME, ZST_MIME, GZ_MIME];
