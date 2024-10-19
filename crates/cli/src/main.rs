mod api_client;

use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
enum Args {
    Push { script_path: PathBuf },
}

fn main() {
    println!("Hello, world!");
}
