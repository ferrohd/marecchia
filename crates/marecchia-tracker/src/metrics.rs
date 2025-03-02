use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::{http::StatusCode, service::Service, Method, Request, Response};
use hyper_util::rt::TokioIo;
use libp2p_metrics::Registry;
use prometheus_client::encoding::text::encode;
use std::future::Future;
use std::net::Ipv6Addr;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::signal::unix::SignalKind;

const METRICS_CONTENT_TYPE: &str = "application/openmetrics-text;charset=utf-8;version=1.0.0";

pub async fn metrics_server(registry: Registry) -> Result<(), std::io::Error> {
    // Serve on localhost.
    let addr = (Ipv6Addr::LOCALHOST, 0);
    let listener = TcpListener::bind(addr).await?;

    let metrics_service = MetricService {
        reg: Arc::new(Mutex::new(registry)),
    };

    let mut sigint = tokio::signal::unix::signal(SignalKind::interrupt())?;
    let mut sigterm = tokio::signal::unix::signal(SignalKind::terminate())?;

    loop {
        tokio::select! {
            _ = sigint.recv() => {
                break;
            }
            _ = sigterm.recv() => {
                break;
            }
            conn = listener.accept() => {
                let (stream, _) = conn?;
                let io = TokioIo::new(stream);
                let service = metrics_service.clone();
                tokio::task::spawn(async move {
                    if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                        println!("Failed to serve connection: {:?}", err);
                    }
                });
            }

        }
    }

    Ok(())
}

type SharedRegistry = Arc<Mutex<Registry>>;

#[derive(Clone)]
pub struct MetricService {
    reg: SharedRegistry,
}

impl Service<Request<Incoming>> for MetricService {
    type Response = Response<String>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        let req_path = req.uri().path();
        let req_method = req.method();

        let resp = if (req_method == Method::GET) && (req_path == "/metrics") {
            let mut response: Response<String> = Response::default();

            response.headers_mut().insert(
                hyper::header::CONTENT_TYPE,
                METRICS_CONTENT_TYPE.try_into().unwrap(),
            );

            let reg = self.reg.clone();
            encode(&mut response.body_mut(), &reg.lock().unwrap()).unwrap();

            *response.status_mut() = StatusCode::OK;

            response
        } else {
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("Not found try localhost:[port]/metrics".to_string())
                .expect("Response should never fail")
        };
        Box::pin(async { Ok(resp) })
    }
}
