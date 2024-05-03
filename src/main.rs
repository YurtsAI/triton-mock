mod server {
    #![allow(clippy::all)]
    tonic::include_proto!("inference");
}

use server::{
    grpc_inference_service_client::GrpcInferenceServiceClient,
    grpc_inference_service_server::{GrpcInferenceService, GrpcInferenceServiceServer},
};

use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    error::Error,
    pin::Pin,
    sync::{Arc, OnceLock},
};

use tonic::{
    transport::{Channel, Server},
    Status,
};

use tokio::sync::Mutex;

use tokio_stream::Stream;

#[derive(serde::Deserialize, serde::Serialize, Default, Debug)]
struct RecordedStream {
    model_config: BTreeMap<String, VecDeque<String>>,
    model_infer: VecDeque<String>,
    model_stream_infer: VecDeque<String>,
    #[serde(skip)]
    model_stream_infer_inputs: VecDeque<String>,
}

#[derive(serde::Deserialize, serde::Serialize, Default, Debug)]
struct RecordedStreams {
    model_map: BTreeMap<String, RecordedStream>,
}

type ResponseStream =
    Pin<Box<dyn Stream<Item = Result<server::ModelStreamInferResponse, Status>> + Send>>;

type ClientMap = HashMap<String, Mutex<GrpcInferenceServiceClient<Channel>>>;

#[derive(Debug, Default)]
struct MockInferenceService {
    recorded_streams: Arc<Mutex<RecordedStreams>>,
}

impl MockInferenceService {
    fn new_with(recorded_streams: Arc<Mutex<RecordedStreams>>) -> Self {
        MockInferenceService { recorded_streams }
    }
}

const CLIENT_PORTS: &[(&[&str], &str)] = &[
    /*
        (
            &[
                "acronym_detector",
                "document_classifier",
                "sentence_embed",
                "ner",
            ],
            "8302",
        ),
        (&["ingestor"], "8303"),
    */
    (&["cross_encoder", "coreference_resolution"], "8304"),
    (&["llama_7b"], "8305"),
    /*
        (&["keybert", "ingestor_vllm"], "8306"),
    */
    (&["mistral_7b_instruct"], "8307"),
];

//const SERVER_PORTS: &[&str] = &["8002", "8003", "8004", "8005", "8006", "8007"];
const SERVER_PORTS: &[&str] = &["8004", "8005", "8007"];

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
    "mistral_7b_instruct",
];

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

    async fn server_ready(
        &self,
        _request: tonic::Request<server::ServerReadyRequest>,
    ) -> std::result::Result<tonic::Response<server::ServerReadyResponse>, tonic::Status> {
        Ok(tonic::Response::new(server::ServerReadyResponse {
            ready: true,
        }))
    }

    async fn model_infer(
        &self,
        request: tonic::Request<server::ModelInferRequest>,
    ) -> std::result::Result<tonic::Response<server::ModelInferResponse>, tonic::Status> {
        let name = request.get_ref().model_name.to_string();
        log::info!("model_infer: '{}'", name);
        if !MODELS.contains(&name.as_ref()) {
            log::error!(
                "model_infer: unknown model '{}', request: {:?}",
                name,
                request
            );
            return Err(tonic::Status::not_found(format!(
                "model_infer: model not found: {}",
                name
            )));
        }
        let request = request.into_inner();
        let mut recorded_stream = self.recorded_streams.lock().await;
        let client_map = GRPC_CLIENT.get();
        if let Some(client_map) = client_map {
            let mut client = client_map.get(&name).unwrap().lock().await;
            let resp = client
                .model_infer(tonic::Request::new(request))
                .await
                .map(|v| {
                    let v = v.into_inner();
                    let model_infer = &mut recorded_stream
                        .model_map
                        .get_mut(&name)
                        .unwrap()
                        .model_infer;
                    model_infer.push_back(serde_json::to_string(&v).unwrap());
                    tonic::Response::new(v)
                })
                .map_err(|e| {
                    log::error!("model_infer: error: {:?}", e);
                    e
                })?;
            log::debug!("model_infer: resp: {resp:?}");
            Ok(resp)
        } else {
            let model_infer = &mut recorded_stream
                .model_map
                .get_mut(&name)
                .unwrap()
                .model_infer;
            let json_resp = model_infer.pop_front();
            if let Some(json_resp) = json_resp {
                let resp: server::ModelInferResponse = serde_json::from_str(&json_resp).unwrap();
                Ok(tonic::Response::new(resp))
            } else {
                Err(tonic::Status::unavailable(
                    "model_infer: no recorded response",
                ))
            }
        }
    }

    async fn model_config(
        &self,
        request: tonic::Request<server::ModelConfigRequest>,
    ) -> std::result::Result<tonic::Response<server::ModelConfigResponse>, tonic::Status> {
        let name = request.get_ref().name.to_string();
        log::info!("model_config: '{}'", name);
        if !MODELS.contains(&name.as_ref()) {
            log::error!(
                "model_config: unknown model '{}', request: {:?}",
                name,
                request
            );
            return Err(tonic::Status::not_found(format!(
                "model_config: model not found: {}",
                name
            )));
        }
        let request = request.into_inner();
        let json = serde_json::to_string(&request).unwrap();
        let mut recorded_stream = self.recorded_streams.lock().await;
        let client_map = GRPC_CLIENT.get();
        if let Some(client_map) = client_map {
            let mut client = client_map.get(&name).unwrap().lock().await;
            let resp = client
                .model_config(tonic::Request::new(request))
                .await
                .map(|v| {
                    let v = v.into_inner();
                    let model_config = &mut recorded_stream
                        .model_map
                        .get_mut(&name)
                        .unwrap()
                        .model_config;
                    let outputs = model_config.entry(json).or_insert(VecDeque::new());
                    outputs.push_back(serde_json::to_string(&v).unwrap());
                    tonic::Response::new(v)
                })
                .map_err(|e| {
                    log::error!("model_config: error: {:?}", e);
                    e
                })?;
            log::debug!("model_config: resp: {resp:?}");
            Ok(resp)
        } else {
            let model_config = &mut recorded_stream
                .model_map
                .get_mut(&name)
                .unwrap()
                .model_config;
            let resp_json = model_config.entry(json).or_default().pop_front();
            if let Some(resp_json) = resp_json {
                let resp: server::ModelConfigResponse = serde_json::from_str(&resp_json).unwrap();
                Ok(tonic::Response::new(resp))
            } else {
                Err(tonic::Status::unavailable(
                    "model_config: no recorded response",
                ))
            }
        }
    }

    async fn model_stream_infer(
        &self,
        request: tonic::Request<tonic::Streaming<server::ModelInferRequest>>,
    ) -> std::result::Result<tonic::Response<Self::ModelStreamInferStream>, tonic::Status> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(4);
        let mut stream = request.into_inner();
        let (client_tx, client_rx) = tokio::sync::oneshot::channel();
        let (recs_tx, recs_rx) = tokio::sync::oneshot::channel();
        let recorded_streams = self.recorded_streams.clone();
        if let Some(client_map) = GRPC_CLIENT.get() {
            tokio::spawn(async move {
                let recorded_streams: Arc<Mutex<RecordedStreams>> = recs_rx.await.unwrap();
                let mut client_tx = Some(client_tx);
                while let Some(model_infer_request) = stream.message().await.unwrap() {
                    if let Some(client_tx) = client_tx.take() {
                        let model_name = model_infer_request.model_name.to_string();
                        let client = client_map.get(&model_name).unwrap().lock().await;
                        client_tx
                            .send((model_name.to_owned(), Some(client.clone())))
                            .unwrap();
                    }
                    let mut recorded_streams = recorded_streams.lock().await;
                    let model_map = recorded_streams
                        .model_map
                        .get_mut(&model_infer_request.model_name)
                        .unwrap();
                    let req_json = serde_json::to_string(&model_infer_request).unwrap();
                    model_map.model_stream_infer_inputs.push_back(req_json);
                    tx.send(model_infer_request).await.unwrap();
                }
            });
        } else {
            tokio::spawn(async move {
                let mut client_tx = Some(client_tx);
                while let Some(model_infer_request) = stream.message().await.unwrap() {
                    if let Some(client_tx) = client_tx.take() {
                        let model_name = model_infer_request.model_name.to_string();
                        client_tx.send((model_name.to_owned(), None)).unwrap();
                    }
                    tx.send(model_infer_request).await.unwrap();
                }
            });
        }
        recs_tx.send(recorded_streams).unwrap();
        let (model_name, client) = client_rx.await.unwrap();
        let (tx2, rx2) = tokio::sync::mpsc::channel(4);
        let recorded_streams = self.recorded_streams.clone();
        let (recs_tx, recs_rx) = tokio::sync::oneshot::channel();
        if let Some(mut client) = client {
            let req_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
            let response = client.model_stream_infer(req_stream).await.unwrap();
            let mut resp_stream = response.into_inner();
            tokio::spawn(async move {
                let recorded_streams: Arc<Mutex<RecordedStreams>> = recs_rx.await.unwrap();
                while let Some(model_infer_resp) = resp_stream.message().await.unwrap() {
                    let mut recorded_streams = recorded_streams.lock().await;
                    let model_map = recorded_streams.model_map.get_mut(&model_name).unwrap();
                    let req_json = model_map.model_stream_infer_inputs.pop_front();
                    if let Some(_req_json) = req_json {
                        let json = serde_json::to_string(&model_infer_resp).unwrap();
                        let outputs = &mut model_map.model_stream_infer;
                        /*
                            .model_stream_infer
                            .entry(req_json)
                            .or_insert(VecDeque::new());
                        */
                        outputs.push_back(json);
                        tx2.send(Ok::<_, tonic::Status>(model_infer_resp))
                            .await
                            .unwrap();
                    } else {
                        tx2.send(Err(tonic::Status::unavailable(
                            "model_stream_infer: no recorded response",
                        )))
                        .await
                        .unwrap();
                    }
                }
            });
        } else {
            tokio::spawn(async move {
                let recorded_streams: Arc<Mutex<RecordedStreams>> = recs_rx.await.unwrap();
                while let Some(_model_infer_req) = rx.recv().await {
                    let mut recorded_streams = recorded_streams.lock().await;
                    let model_map = recorded_streams.model_map.get_mut(&model_name).unwrap();
                    //let req_json = serde_json::to_string(&model_infer_req).unwrap();
                    let resp_json = model_map
                        .model_stream_infer
                        /*
                        .get_mut(&req_json)
                        .unwrap_or(&mut VecDeque::new())
                        */
                        .pop_front();
                    if let Some(resp_json) = resp_json {
                        let resp = serde_json::from_str(&resp_json).unwrap();
                        tx2.send(Ok::<_, tonic::Status>(resp)).await.unwrap();
                    } else {
                        tx2.send(Err(tonic::Status::unavailable(
                            "model_stream_infer: no recorded response",
                        )))
                        .await
                        .unwrap();
                    }
                }
            });
        }
        recs_tx.send(recorded_streams).unwrap();
        Ok(tonic::Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(rx2),
        )))
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

#[derive(clap::Parser, Debug)]
struct CliOptions {
    #[clap(long)]
    record: bool,
    #[clap(long, default_value = "host.docker.internal")]
    remote_host: String,
    #[clap(long, default_value = "0")]
    suffix: String,
}

fn recording_filename(suffix: &str) -> String {
    format!("triton-mock-recording-{}.json.gz", suffix)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    use clap::Parser;

    env_logger::init();
    log::info!("Starting server...");

    let mut client_map = ClientMap::new();
    let cli_options = CliOptions::parse();

    let pid = std::process::id();
    let pid_fname = "/tmp/triton-mock-server.pid";

    std::fs::write(pid_fname, format!("{}", pid))?;

    let recorded_streams = if cli_options.record {
        let mut recorded_streams = RecordedStreams::default();
        for (models, port) in CLIENT_PORTS {
            for model in *models {
                let address = format!("http://{}:{}", cli_options.remote_host, port);
                log::info!("Connecting to remote gRPC endpoint: {address}");
                let client = Mutex::new(GrpcInferenceServiceClient::connect(address).await?);
                client_map.insert(model.to_string(), client);
                recorded_streams
                    .model_map
                    .insert(model.to_string(), RecordedStream::default());
            }
        }
        GRPC_CLIENT.set(client_map).unwrap();
        recorded_streams
    } else {
        let fname = recording_filename(&cli_options.suffix);
        let fin = std::fs::File::open(fname).unwrap();
        let fin_gz = flate2::read::GzDecoder::new(fin);
        serde_json::from_reader(fin_gz).unwrap()
    };

    let mut join_set = tokio::task::JoinSet::new();
    let recorded_streams = Arc::new(Mutex::new(recorded_streams));

    for port in SERVER_PORTS {
        let address = format!("0.0.0.0:{}", port).parse().unwrap();
        let service = MockInferenceService::new_with(recorded_streams.clone());
        let port = Server::builder()
            .add_service(GrpcInferenceServiceServer::new(service))
            .serve_with_shutdown(address, async {
                tokio::signal::ctrl_c()
                    .await
                    .expect("failed to install CTRL+C signal handler");
            });
        join_set.spawn(port);
    }

    if let Some(res) = join_set.join_next().await {
        log::warn!("Shutdown was signaled: {:?}", res);
    }

    join_set.abort_all();

    if cli_options.record {
        let recorded_streams = recorded_streams.lock().await;
        let fname = recording_filename(&cli_options.suffix);
        let fout = std::fs::File::create(fname).unwrap();
        let fout_gz = flate2::write::GzEncoder::new(fout, flate2::Compression::default());
        serde_json::to_writer(fout_gz, &*recorded_streams).unwrap();
    }

    Ok(())
}
