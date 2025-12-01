fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:warning=Building proto files...");
    println!("cargo:rerun-if-changed=proto/fx.proto");

    // Use bundled protoc on Windows if not set
    #[cfg(windows)]
    {
        if std::env::var("PROTOC").is_err() {
            let protoc_path = protoc_bin_vendored::protoc_bin_path().unwrap();
            std::env::set_var("PROTOC", protoc_path);
        }
    }

    // Generate to a fixed location for Rust Analyzer compatibility
    let out_dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let generated_dir = out_dir.join("src/generated");
    std::fs::create_dir_all(&generated_dir)?;

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir(&generated_dir)
        .compile(&["proto/fx.proto"], &["proto/"])?;

    println!("cargo:warning=Proto files built successfully");
    Ok(())
}
