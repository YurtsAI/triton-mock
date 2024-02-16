fn main() -> Result<(), Box<dyn std::error::Error>> {
    //tonic_build::compile_protos("protos/grpc_service.proto")?;

    tonic_build::configure()
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile(&["protos/grpc_service.proto"], &["protos"])
        .unwrap();

    Ok(())
}
