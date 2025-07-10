// contracts-artifacts/build.rs

use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let contracts_dir = manifest_dir.join("../contracts");
    let out_dir = contracts_dir.join("out");

    println!("cargo:warning=📦 contracts_dir = {:?}", contracts_dir);
    println!("cargo:warning=📦 out_dir       = {:?}", out_dir);

    if let Err(e) = fs::create_dir_all(&out_dir) {
        println!("cargo:warning=❌ failed to mkdir {:?}: {}", out_dir, e);
    }

    let status = Command::new("make")
        .arg("build")
        .current_dir(&contracts_dir)
        .status()
        .expect("failed to run `make build` in contracts-artifacts");
    if !status.success() {
        panic!("`make build` failed");
    }

    match fs::read_dir(&out_dir) {
        Ok(rd) => {
            for entry in rd.flatten() {
                println!("cargo:warning=📄 found entry: {:?}", entry.path());
                // if it's a directory, list one level deeper:
                if entry.path().is_dir() {
                    if let Ok(rd2) = fs::read_dir(entry.path()) {
                        for e2 in rd2.flatten() {
                            println!("cargo:warning=    └── {:?}", e2.path());
                        }
                    }
                }
            }
        }
        Err(e) => println!("cargo:warning=❌ read_dir failed on {:?}: {}", out_dir, e),
    }

    println!(
        "cargo:rerun-if-changed={}/Makefile",
        contracts_dir.display()
    );
    println!(
        "cargo:rerun-if-changed={}/contracts/**/*.sol",
        contracts_dir.display()
    );
    println!("cargo:rerun-if-changed={}", out_dir.display());
}
