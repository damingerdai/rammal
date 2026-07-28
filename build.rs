use std::error::Error;
use vergen_git2::{Build, Cargo, Emitter, Git2, Rustc, Sysinfo};

fn main() -> Result<(), Box<dyn Error>> {
   let build = Build::all_build();
let cargo = Cargo::all_cargo();
let git2 = Git2::all_git();
let rustc = Rustc::all_rustc();
let si = Sysinfo::all_sysinfo();

    Emitter::default()
        .add_instructions(&build)?
        .add_instructions(&cargo)?
        .add_instructions(&git2)?
        .add_instructions(&rustc)?
        .add_instructions(&si)?
        .emit()?;

    Ok(())
}