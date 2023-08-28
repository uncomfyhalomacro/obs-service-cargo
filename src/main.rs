use clap::Parser;
use obs_service_cargo_vendor_rs::Opts;

fn main() {
    let args = Opts::parse();
    let srcdir = args.srcdir;
    println!("{:?}", srcdir);
}
