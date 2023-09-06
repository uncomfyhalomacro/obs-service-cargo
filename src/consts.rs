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
