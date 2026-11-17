// Build script: embeds app.ico and app.manifest into the executable.
// Uses rc.exe from the Windows SDK. If it is not found, the build still
// succeeds; the app then falls back to loading app.ico from its folder.

use std::path::{Path, PathBuf};
use std::process::Command;

fn version_key(name: &str) -> Vec<u32> {
    name.split('.')
        .map(|p| p.parse::<u32>().unwrap_or(0))
        .collect()
}

fn find_rc() -> Option<PathBuf> {
    // 1) Try PATH.
    if let Ok(out) = Command::new("where.exe").arg("rc.exe").output() {
        if out.status.success() {
            let text = String::from_utf8_lossy(&out.stdout);
            if let Some(line) = text.lines().find(|l| !l.trim().is_empty()) {
                let p = PathBuf::from(line.trim());
                if p.is_file() {
                    return Some(p);
                }
            }
        }
    }

    // 2) Search the Windows 10/11 SDK, newest version first.
    let mut roots: Vec<PathBuf> = Vec::new();
    for var in ["ProgramFiles(x86)", "ProgramFiles"] {
        if let Ok(v) = std::env::var(var) {
            roots.push(PathBuf::from(v).join("Windows Kits").join("10").join("bin"));
        }
    }
    roots.push(PathBuf::from(r"C:\Program Files (x86)\Windows Kits\10\bin"));

    for root in roots {
        let Ok(entries) = std::fs::read_dir(&root) else {
            continue;
        };
        let mut dirs: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        dirs.sort_by_key(|p| {
            let n = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
            version_key(n)
        });
        for dir in dirs.iter().rev() {
            for arch in ["x64", "x86", "arm64"] {
                let candidate = dir.join(arch).join("rc.exe");
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }
    None
}

fn main() {
    println!("cargo:rerun-if-changed=app.rc");
    println!("cargo:rerun-if-changed=app.manifest");
    println!("cargo:rerun-if-changed=app.ico");

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let res_path = out_dir.join("app.res");

    let Some(rc) = find_rc() else {
        println!("cargo:warning=rc.exe not found; building without embedded icon/manifest");
        return;
    };

    let status = Command::new(&rc)
        .current_dir(&manifest_dir)
        .arg("/nologo")
        .arg("/fo")
        .arg(&res_path)
        .arg("app.rc")
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("cargo:rustc-link-arg={}", res_path.display());
        }
        _ => {
            println!("cargo:warning=rc.exe failed; building without embedded icon/manifest");
        }
    }

    // Make sure the resource file is not treated as a stale artifact.
    if !Path::new(&res_path).exists() {
        println!("cargo:warning=resource file was not produced");
    }
}
