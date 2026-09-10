//! After a native CLI build, copy the executable to the repository root.
//!
//! Cargo has no post-link hook, so this script starts a helper process that
//! waits for the binary to appear and then copies it.

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if env::var("CARGO_FEATURE_CLI").is_err() {
        return;
    }

    let out = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let Some(profile_dir) = out.ancestors().nth(3) else {
        return;
    };
    let package = env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "deck_gen".into());
    let exe_name = if cfg!(windows) {
        format!("{package}.exe")
    } else {
        package
    };
    let exe = profile_dir.join(&exe_name);
    let crate_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let Some(repo) = crate_dir.parent() else {
        return;
    };
    let dest = repo.join(&exe_name);
    spawn_copy_when_ready(exe, dest);
}

fn spawn_copy_when_ready(exe: PathBuf, dest: PathBuf) {
    let exe_s = exe.display().to_string();
    let dest_s = dest.display().to_string();
    if cfg!(windows) {
        let script = format!(
            "$exe = '{}'; $dest = '{}'; $deadline = (Get-Date).AddMinutes(2); \
             while ((Get-Date) -lt $deadline) {{ \
               if (Test-Path -LiteralPath $exe) {{ \
                 try {{ Copy-Item -LiteralPath $exe -Destination $dest -Force; exit 0 }} catch {{}} \
               }}; Start-Sleep -Milliseconds 400 \
             }}",
            exe_s.replace('\'', "''"),
            dest_s.replace('\'', "''"),
        );
        let _ = Command::new("powershell")
            .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &script])
            .spawn();
    } else {
        let script = format!(
            "exe='{exe_s}'; dest='{dest_s}'; \
             for i in $(seq 1 300); do \
               if [ -f \"$exe\" ]; then cp -f \"$exe\" \"$dest\" && exit 0; fi; \
               sleep 0.4; \
             done"
        );
        let _ = Command::new("sh").args(["-c", &script]).spawn();
    }
}
