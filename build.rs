use std::{fs::File, io::Write, path::Path};

// generated by `sqlx migrate build-script`
fn main() {
    // trigger recompilation when a new migration is added
    println!("cargo:rerun-if-changed=migrations");

    // Build scss files into a css bundle
    println!("cargo:rerun-if-changed=styles");
    let css: &str = grass::include!("styles/main.scss");
    let bundle_path = Path::new("assets/style.css");
    let mut bundle = File::create(bundle_path).unwrap();
    bundle.write_all(css.as_bytes()).unwrap();
}