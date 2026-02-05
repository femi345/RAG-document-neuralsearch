fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
        .join("proto");

    tonic_build::configure()
        .build_server(false)
        .compile_protos(
            &[proto_dir.join("ml_service.proto")],
            &[&proto_dir],
        )?;
    Ok(())
}
