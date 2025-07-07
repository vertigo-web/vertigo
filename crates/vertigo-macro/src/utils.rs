pub fn build_profile() -> &'static str {
    env!("VERTIGO_PROFILE")
}

pub fn release_build() -> bool {
    build_profile() == "release"
}
