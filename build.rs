use std::error::Error;
use vergen_git2::{Build, Cargo, Emitter, Git2, Rustc, Sysinfo};

fn main() -> Result<(), Box<dyn Error>> {
    // Fallbacks so the build still succeeds when .git is unavailable
    // (e.g. building from a source tarball or a docker context without .git).
    println!("cargo:rustc-env=VERGEN_GIT_DESCRIBE=unknown");
    println!("cargo:rustc-env=VERGEN_GIT_SHA=unknown");
    println!("cargo:rustc-env=VERGEN_GIT_COMMIT_DATE=unknown");
    println!("cargo:rustc-env=VERGEN_GIT_BRANCH=unknown");
    println!("cargo:rerun-if-changed=build.rs");

    let build = Build::all_build();
    let cargo = Cargo::all_cargo();
    let git2 = Git2::all_git();
    let rustc = Rustc::all_rustc();
    let si = Sysinfo::all_sysinfo();

    let mut emitter = Emitter::default();
    emitter.fail_on_error();
    let _ = emitter.add_instructions(&build);
    let _ = emitter.add_instructions(&cargo);
    let _ = emitter.add_instructions(&git2);
    let _ = emitter.add_instructions(&rustc);
    let _ = emitter.add_instructions(&si);
    let _ = emitter.emit();

    // Values injected via docker build-args / env take precedence over
    // whatever vergen produced (or failed to produce) when .git is absent.
    for key in [
        "VERGEN_GIT_DESCRIBE",
        "VERGEN_GIT_SHA",
        "VERGEN_GIT_COMMIT_DATE",
    ] {
        if let Ok(val) = std::env::var(key) {
            if !val.is_empty() {
                println!("cargo:rustc-env={key}={val}");
            }
        }
    }

    Ok(())
}