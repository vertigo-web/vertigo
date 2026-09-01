use std::env;

fn main() {
    // Save PROFILE to env variable for use by bundling macros. Directly used by get_target_dir function.
    export_var("PROFILE");

    // No `rerun-if-*` directive here on purpose: a PROFILE change already yields a
    // different build-script unit, and emitting any of them would narrow cargo from
    // tracking every source of this crate down to the listed paths only.
}

fn export_var(name: &str) {
    let value = &env::var(name).unwrap_or_else(|err| {
        panic!("Can't read {name} env variable in vertigo-macro build script: {err}")
    });
    println!("cargo:rustc-env=VERTIGO_{name}={value}");
}
