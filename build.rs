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

    Emitter::default()
        .fail_on_error()
        .add_instructions(&build)?
        .add_instructions(&cargo)?
        .add_instructions(&git2)?
        .add_instructions(&rustc)?
        .add_instructions(&si)?
        .emit()?;

    Ok(())
}