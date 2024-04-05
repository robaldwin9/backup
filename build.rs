use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Get the OUT_DIR environment variable where Cargo places build artifacts
    let out_dir = env::var("OUT_DIR").unwrap();
    let target_dir = Path::new(&out_dir).ancestors().nth(3).unwrap().to_path_buf();

    // Define the source file path
    let source_file = Path::new(env::var("CARGO_MANIFEST_DIR").unwrap().as_str())
        .join("resources/backup.ini");

    // Define the destination file path
    let destination_file = target_dir.join("backup.ini");

    // Copy the file from the source to the destination
    fs::copy(&source_file, &destination_file).expect("Failed to copy file");

    println!("cargo:rerun-if-changed=src/resources/backup.ini");
}