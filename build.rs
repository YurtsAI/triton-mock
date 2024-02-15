fn main() -> Result<(), Box<dyn std::error::Error>> {
    //tonic_build::compile_protos("protos/grpc_service.proto")?;

    tonic_build::configure()
        .type_attribute(
            "inference.ModelInferResponse",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "inference.ModelInferResponse.InferOutputTensor",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "inference.InferTensorContents",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "inference.InferParameter",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "inference.ModelRepositoryParameter.parameter_choice",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "inference.InferParameter.parameter_choice",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .compile(&["protos/grpc_service.proto"], &["protos"])
        .unwrap();

    Ok(())
}
