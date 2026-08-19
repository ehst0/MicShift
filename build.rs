use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn find_rc(target_arch: &str) -> Option<PathBuf> {
    if let Ok(path) = env::var("PATH") {
        for dir in env::split_paths(&path) {
            let candidate = dir.join("rc.exe");
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    let program_files = env::var_os("ProgramFiles(x86)")?;
    let bin = PathBuf::from(program_files).join("Windows Kits/10/bin");
    let arch = match target_arch {
        "x86" => "x86",
        "aarch64" => "arm64",
        _ => "x64",
    };
    let mut versions: Vec<PathBuf> = fs::read_dir(bin)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path().join(arch).join("rc.exe"))
        .filter(|path| path.is_file())
        .collect();
    versions.sort();
    versions.pop()
}

fn main() {
    println!("cargo:rerun-if-changed=resources/micshift.ico");
    println!("cargo:rerun-if-changed=resources/micshift.rc");
    println!("cargo:rerun-if-changed=resources/micshift.manifest");
    println!("cargo:rerun-if-changed=resources/banner.bmp");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is missing"));
    let output = out_dir.join("micshift.res");
    let rc = find_rc(&env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default())
        .expect("Windows resource compiler (rc.exe) was not found");
    let status = Command::new(rc)
        .args(["/nologo", "/fo"])
        .arg(&output)
        .arg(Path::new("resources/micshift.rc"))
        .status()
        .expect("failed to start rc.exe");
    assert!(
        status.success(),
        "rc.exe failed to compile MicShift resources"
    );

    for binary in ["MicShift", "MicShiftConsole"] {
        println!("cargo:rustc-link-arg-bin={binary}={}", output.display());
    }
}
