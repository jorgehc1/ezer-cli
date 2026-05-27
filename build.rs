fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let sdk_manifest = std::path::Path::new(&manifest_dir)
        .parent()
        .unwrap()
        .join("ezerdesk-sdk")
        .join("Cargo.toml");

    let version = if sdk_manifest.exists() {
        let content = std::fs::read_to_string(sdk_manifest).unwrap();
        let value: toml::Value = toml::from_str(&content).unwrap();
        value["package"]["version"]
            .as_str()
            .unwrap_or("0.1.3")
            .to_string()
    } else {
        "0.1.3".to_string()
    };

    println!("cargo:rustc-env=EZERDESK_SDK_VERSION={}", version);
    println!("cargo:rerun-if-changed=../ezerdesk-sdk/Cargo.toml");
}
