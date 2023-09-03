use hyper::server::conn::AddrIncoming;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use hyper::{Method, StatusCode};
use hyper_rustls::TlsAcceptor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Config {
    #[structopt(short, long, parse(from_os_str))]
    notes_dir: std::path::PathBuf,
}


fn main() {
    let config = Config::from_args();
    if !config.notes_dir.exists() {
        create_dir_all(&config.notes_dir).unwrap();
    }

    // Serve an echo service over HTTPS, with proper error handling.
    if let Err(e) = run_server(config) {
        eprintln!("FAILED: {}", e);
        std::process::exit(1);
    }
}


#[tokio::main]
async fn run_server(config: Config) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let port = 3000;
    let addr = format!("127.0.0.1:{}", port).parse()?;

    
    let notes_dir = Arc::new(Mutex::new(config.notes_dir.clone()));
    let notes_map = Arc::new(Mutex::new(HashMap::<String, String>::new()));


    // Load public certificate.
    // let certs = load_certs("localhost.crt")?;
    let certs = load_certs("onboarder+1.pem")?;
    // Load private key.
    // let key = load_private_key("localhost.key")?;
    let key = load_private_key("onboarder+1-key.pem")?;
    // Build TLS configuration.

    // Create a TCP listener via tokio.
    let incoming = AddrIncoming::bind(&addr)?;
    let acceptor = TlsAcceptor::builder()
        .with_single_cert(certs, key)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("{}", e)))?
        .with_all_versions_alpn()
        .with_incoming(incoming);
    let service = make_service_fn(move |_| {
                let notes_dir = notes_dir.clone();
                let notes_map = notes_map.clone();
                async move {
                    Ok::<_, Infallible>(service_fn(move |req| {
                        let notes_dir = notes_dir.clone();
                        let notes_map = notes_map.clone();
                        handle(req, notes_dir, notes_map)
                    }))
                }
            });

//     let port = 3000;
//     let addr = format!("127.0.0.1:{}", port).parse()?;
    
//     let incoming = AddrIncoming::bind(&addr)?;
//     // let server = Server::bind(&addr).serve(make_svc);

    let server = Server::builder(acceptor).serve(service);

    // Run the future, keep going until an error occurs.
    println!("Starting to serve on https://{}.", addr);
    server.await?;
    Ok(())
}


#[derive(Serialize, Deserialize, Debug)]
struct Note {
    id: String,
    content: String,
}

fn get_path_for_note_id(id: &str, notes_dir: &PathBuf, notes_map: &mut HashMap<String, String>) -> PathBuf {
    if let Some(path) = notes_map.get(id) {
        return PathBuf::from(path);
    }
    
    let invalid_chars: Vec<char> = vec!['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    let sanitized_id = id.chars()
        .map(|c| if invalid_chars.contains(&c) { '_' } else { c })
        .collect::<String>();
    
    let file_path = notes_dir.join(format!("{}.txt", &sanitized_id));

    notes_map.insert(id.to_string(), file_path.to_str().unwrap().to_string());

    file_path
}

async fn handle(
    req: Request<Body>,
    notes_dir: Arc<Mutex<PathBuf>>,
    notes_map: Arc<Mutex<HashMap<String, String>>>
) -> Result<Response<Body>, Infallible> {
    println!("{} {}", req.method(), req.uri().path());
    let mut res = match (req.method(), req.uri().path()) {
        (&Method::OPTIONS, _) => {
            let res: Response<Body> = Response::new("".into());
            Ok::<Response<Body>, Infallible>(res)
        },
        (&Method::GET, "/healthcheck") => {
            let mut res = Response::default();
            *res.status_mut() = StatusCode::OK;
            Ok(res)
        },
        (&Method::POST, "/set_note") => {
            let dir = notes_dir.lock().await;
            let mut map = notes_map.lock().await;

            let whole_body = hyper::body::to_bytes(req.into_body()).await.unwrap();
            let note: Note = serde_json::from_slice(&whole_body).unwrap();

            println!("Note: {:?}", note);

            let file_path = get_path_for_note_id(&note.id, &dir, &mut map);

            let mut file = OpenOptions::new().create(true).write(true).truncate(true).open(&file_path).unwrap();
            file.write_all(note.content.as_bytes()).unwrap();

            let res: Response<Body> = Response::new("Note set".into());
            Ok(res)
        }
        (&Method::GET, "/get_note") => {
            // TODO: Handle GET requests
            Ok(Response::new("GET not implemented yet".into()))
        }
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }?;
    let headers = res.headers_mut();
    headers.insert("Access-Control-Allow-Origin", "https://www.youtube.com".parse().unwrap());
    headers.insert("Access-Control-Allow-Methods", "GET, POST, OPTIONS".parse().unwrap());
    headers.insert("Access-Control-Allow-Headers", "Content-Type".parse().unwrap());
    Ok(res)
}


// Load public certificate from file.
fn load_certs(filename: &str) -> std::io::Result<Vec<rustls::Certificate>> {
    // Open certificate file.
    let certfile = std::fs::File::open(filename)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("failed to open {} (did you run gen-certs.ps1?): {}", filename, e)))?;
    let mut reader = std::io::BufReader::new(certfile);

    // Load and return certificate.
    let certs = rustls_pemfile::certs(&mut reader)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "failed to load certificate"))?;
    Ok(certs
        .into_iter()
        .map(rustls::Certificate)
        .collect())
}

// Load private key from file.
fn load_private_key(filename: &str) -> std::io::Result<rustls::PrivateKey> {
    // Open keyfile.
    let keyfile = std::fs::File::open(filename)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("failed to open {} (did you run gen-certs.ps1?): {}", filename, e)))?;
    let mut reader = std::io::BufReader::new(keyfile);

    // Load and return a single private key.
    let keys = rustls_pemfile::pkcs8_private_keys(&mut reader)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "failed to load private key"))?;
    if keys.len() != 1 {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("expected a single private key, got {} instead", keys.len())));
    }

    Ok(rustls::PrivateKey(keys[0].clone()))
}