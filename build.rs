fn main() {
    #[cfg(feature = "server")]
    println!("cargo:rerun-if-changed=migrations");
}
