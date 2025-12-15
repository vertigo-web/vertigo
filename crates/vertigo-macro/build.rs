use std::env;

fn main() {
    // Save PROFILE to env variable for use by bundling macros. Directly used by get_target_dir function.
    export_var("PROFILE");
    println!("cargo:rerun-if-changed-env=PROFILE")
}

fn export_var(name: &str) {
    let value = &env::var(name).unwrap_or_else(|err| {
        panic!("Can't read {name} env variable in vertigo-macro build script: {err}")
    });
    println!("cargo:rustc-env=VERTIGO_{name}={value}");
}
