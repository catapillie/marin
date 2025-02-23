use build_print as build;

const MARIN_STD_PATH: &str = "std/";

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed={}", MARIN_STD_PATH);

    let target_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let profile = std::env::var("PROFILE").unwrap();
    let output_path = std::path::Path::new(&target_dir)
        .join("target")
        .join(&profile);

    build::custom_println!(
        "Marin build",
        green,
        "refreshing included std library ({})",
        output_path.display()
    );

    let output_dir = output_path.join(MARIN_STD_PATH);
    fs_extra::dir::create(&output_dir, true)
        .expect("couldn't recreate marin std library directory");

    let options = fs_extra::dir::CopyOptions::new().overwrite(true);
    fs_extra::dir::copy(MARIN_STD_PATH, &output_path, &options)
        .expect("failed to copy marin std to output directory");
}
