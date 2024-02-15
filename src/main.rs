mod server {
    tonic::include_proto!("inference");
}

use server::{
    grpc_inference_service_client::GrpcInferenceServiceClient,
    grpc_inference_service_server::{GrpcInferenceService, GrpcInferenceServiceServer},
};

use std::{collections::HashMap, error::Error, pin::Pin, sync::OnceLock};
use tonic::{
    transport::{Channel, Server},
    Status,
};

use tokio_stream::Stream;

#[derive(Debug, Default)]
struct MockInferenceService {}

type ResponseStream =
    Pin<Box<dyn Stream<Item = Result<server::ModelStreamInferResponse, Status>> + Send>>;

const CLIENT_PORTS: &[(&[&str], &str)] = &[
    (
        &[
            "acronym_detector",
            "document_classifier",
            "sentence_embed",
            "ner",
            "keybert",
        ],
        "8302",
    ),
    (&["ingestor"], "8303"),
    (&["coreference_resolution"], "8304"),
    (&["llama_7b"], "8305"),
    (&["ingestor_vllm"], "8306"),
    (&["mistral_7b_instruct"], "8307"),
];

const SERVER_PORTS: &[&str] = &["8002", "8003", "8004", "8005", "8006", "8007"];

const MODELS: &[&str] = &[
    "acronym_detector",
    "document_classifier",
    "sentence_embed",
    "ner",
    "keybert",
    "ingestor",
    "coreference_resolution",
    "cross_encoder",
    "llama_7b",
];

type ClientMap = HashMap<String, GrpcInferenceServiceClient<Channel>>;

static GRPC_CLIENT: OnceLock<ClientMap> = OnceLock::new();

#[tonic::async_trait]
impl GrpcInferenceService for MockInferenceService {
    type ModelStreamInferStream = ResponseStream;

    async fn server_live(
        &self,
        _request: tonic::Request<server::ServerLiveRequest>,
    ) -> std::result::Result<tonic::Response<server::ServerLiveResponse>, tonic::Status> {
        Ok(tonic::Response::new(server::ServerLiveResponse {
            live: true,
        }))
    }

    async fn model_ready(
        &self,
        request: tonic::Request<server::ModelReadyRequest>,
    ) -> std::result::Result<tonic::Response<server::ModelReadyResponse>, tonic::Status> {
        let name = &request.get_ref().name.as_ref();
        if MODELS.contains(name) {
            log::info!("model_ready: {:?}", request);
            Ok(tonic::Response::new(server::ModelReadyResponse {
                ready: true,
            }))
        } else {
            log::error!(
                "model_ready: unknown model '{}', request: {:?}",
                name,
                request
            );
            Ok(tonic::Response::new(server::ModelReadyResponse {
                ready: false,
            }))
        }
    }

    async fn model_infer(
        &self,
        request: tonic::Request<server::ModelInferRequest>,
    ) -> std::result::Result<tonic::Response<server::ModelInferResponse>, tonic::Status> {
        log::warn!("Not implemented: model_infer: {:?}", request);
        return Err(tonic::Status::unimplemented("model_infer not implemented"));
    }

    async fn server_ready(
        &self,
        _request: tonic::Request<server::ServerReadyRequest>,
    ) -> std::result::Result<tonic::Response<server::ServerReadyResponse>, tonic::Status> {
        Ok(tonic::Response::new(server::ServerReadyResponse {
            ready: true,
        }))
    }

    async fn model_config(
        &self,
        request: tonic::Request<server::ModelConfigRequest>,
    ) -> std::result::Result<tonic::Response<server::ModelConfigResponse>, tonic::Status> {
        let name = &request.get_ref().name.as_ref();
        log::info!("model_config: '{}'", name);
        if !MODELS.contains(name) {
            log::error!(
                "model_config: unknown model '{}', request: {:?}",
                name,
                request
            );
            return Err(tonic::Status::not_found(format!(
                "model not found: {}",
                name
            )));
        }
        let config = server::ModelConfig {
            model_transaction_policy: Some(server::ModelTransactionPolicy { decoupled: false }),
            ..Default::default()
        };
        Ok(tonic::Response::new(server::ModelConfigResponse {
            config: Some(config),
        }))
    }

    async fn log_settings(
        &self,
        request: tonic::Request<server::LogSettingsRequest>,
    ) -> std::result::Result<tonic::Response<server::LogSettingsResponse>, tonic::Status> {
        log::warn!("Not implemented: log_settings: {:?}", request);
        return Err(tonic::Status::unimplemented("log_settings not implemented"));
    }

    async fn trace_setting(
        &self,
        request: tonic::Request<server::TraceSettingRequest>,
    ) -> std::result::Result<tonic::Response<server::TraceSettingResponse>, tonic::Status> {
        log::warn!("Not implemented: trace_setting: {:?}", request);
        return Err(tonic::Status::unimplemented(
            "trace_setting not implemented",
        ));
    }

    async fn model_metadata(
        &self,
        request: tonic::Request<server::ModelMetadataRequest>,
    ) -> std::result::Result<tonic::Response<server::ModelMetadataResponse>, tonic::Status> {
        log::warn!("Not implemented: model_metadata: {:?}", request);
        return Err(tonic::Status::unimplemented(
            "model_metadata not implemented",
        ));
    }

    async fn server_metadata(
        &self,
        request: tonic::Request<server::ServerMetadataRequest>,
    ) -> std::result::Result<tonic::Response<server::ServerMetadataResponse>, tonic::Status> {
        log::warn!("Not implemented: server_metadata: {:?}", request);
        return Err(tonic::Status::unimplemented(
            "server_metadata not implemented",
        ));
    }

    async fn model_statistics(
        &self,
        request: tonic::Request<server::ModelStatisticsRequest>,
    ) -> std::result::Result<tonic::Response<server::ModelStatisticsResponse>, tonic::Status> {
        log::warn!("Not implemented: model_statistics: {:?}", request);
        return Err(tonic::Status::unimplemented(
            "model_statistics not implemented",
        ));
    }

    async fn repository_index(
        &self,
        request: tonic::Request<server::RepositoryIndexRequest>,
    ) -> std::result::Result<tonic::Response<server::RepositoryIndexResponse>, tonic::Status> {
        log::warn!("Not implemented: repository_index: {:?}", request);
        return Err(tonic::Status::unimplemented(
            "repository_index not implemented",
        ));
    }

    async fn model_stream_infer(
        &self,
        request: tonic::Request<tonic::Streaming<server::ModelInferRequest>>,
    ) -> std::result::Result<tonic::Response<Self::ModelStreamInferStream>, tonic::Status> {
        log::warn!("Not implemented: model_stream_infer: {:?}", request);
        return Err(tonic::Status::unimplemented(
            "model_stream_infer not implemented",
        ));
    }

    async fn repository_model_load(
        &self,
        request: tonic::Request<server::RepositoryModelLoadRequest>,
    ) -> std::result::Result<tonic::Response<server::RepositoryModelLoadResponse>, tonic::Status>
    {
        log::warn!("Not implemented: repository_model_load: {:?}", request);
        return Err(tonic::Status::unimplemented(
            "repository_model_load not implemented",
        ));
    }

    async fn repository_model_unload(
        &self,
        request: tonic::Request<server::RepositoryModelUnloadRequest>,
    ) -> std::result::Result<tonic::Response<server::RepositoryModelUnloadResponse>, tonic::Status>
    {
        log::warn!("Not implemented: repository_model_unload: {:?}", request);
        return Err(tonic::Status::unimplemented(
            "repository_model_unload not implemented",
        ));
    }

    async fn cuda_shared_memory_status(
        &self,
        request: tonic::Request<server::CudaSharedMemoryStatusRequest>,
    ) -> std::result::Result<tonic::Response<server::CudaSharedMemoryStatusResponse>, tonic::Status>
    {
        log::warn!("Not implemented: cuda_shared_memory_status: {:?}", request);
        return Err(tonic::Status::unimplemented(
            "cuda_shared_memory_status not implemented",
        ));
    }

    async fn system_shared_memory_status(
        &self,
        request: tonic::Request<server::SystemSharedMemoryStatusRequest>,
    ) -> std::result::Result<tonic::Response<server::SystemSharedMemoryStatusResponse>, tonic::Status>
    {
        log::warn!(
            "Not implemented: system_shared_memory_status: {:?}",
            request
        );
        return Err(tonic::Status::unimplemented(
            "system_shared_memory_status not implemented",
        ));
    }

    async fn cuda_shared_memory_register(
        &self,
        request: tonic::Request<server::CudaSharedMemoryRegisterRequest>,
    ) -> std::result::Result<tonic::Response<server::CudaSharedMemoryRegisterResponse>, tonic::Status>
    {
        log::warn!("Not implemented: cuda_shared_memory_status: {:?}", request);
        return Err(tonic::Status::unimplemented(
            "cuda_shared_memory_status not implemented",
        ));
    }

    async fn system_shared_memory_register(
        &self,
        request: tonic::Request<server::SystemSharedMemoryRegisterRequest>,
    ) -> std::result::Result<
        tonic::Response<server::SystemSharedMemoryRegisterResponse>,
        tonic::Status,
    > {
        log::warn!(
            "Not implemented: system_shared_memory_status: {:?}",
            request
        );
        return Err(tonic::Status::unimplemented(
            "system_shared_memory_status not implemented",
        ));
    }

    async fn cuda_shared_memory_unregister(
        &self,
        request: tonic::Request<server::CudaSharedMemoryUnregisterRequest>,
    ) -> std::result::Result<
        tonic::Response<server::CudaSharedMemoryUnregisterResponse>,
        tonic::Status,
    > {
        log::warn!("Not implemented: cuda_shared_memory_status: {:?}", request);
        return Err(tonic::Status::unimplemented(
            "cuda_shared_memory_status not implemented",
        ));
    }

    async fn system_shared_memory_unregister(
        &self,
        request: tonic::Request<server::SystemSharedMemoryUnregisterRequest>,
    ) -> std::result::Result<
        tonic::Response<server::SystemSharedMemoryUnregisterResponse>,
        tonic::Status,
    > {
        log::warn!(
            "Not implemented: system_shared_memory_status: {:?}",
            request
        );
        return Err(tonic::Status::unimplemented(
            "system_shared_memory_status not implemented",
        ));
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    log::info!("Starting server...");

    let mut client_map = ClientMap::new();

    for (models, port) in CLIENT_PORTS {
        for model in *models {
            let address = format!("http://0.0.0.0:{}", port);
            let client = GrpcInferenceServiceClient::connect(address).await?;
            client_map.insert(model.to_string(), client);
        }
    }

    GRPC_CLIENT.set(client_map).unwrap();

    let mut set = tokio::task::JoinSet::new();

    for port in SERVER_PORTS {
        let address = format!("0.0.0.0:{}", port).parse().unwrap();
        let service = MockInferenceService::default();
        let port = Server::builder()
            .add_service(GrpcInferenceServiceServer::new(service))
            .serve(address);
        set.spawn(port);
    }

    while let Some(res) = set.join_next().await {
        log::warn!("A server has stopped: {:?}", res);
    }

    set.abort_all();

    Ok(())
}
