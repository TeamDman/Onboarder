use hyper::server::conn::AddrIncoming;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use hyper::{Method, StatusCode};
use hyper_rustls::TlsAcceptor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::fs::{create_dir_all, OpenOptions};
use std::io::{Write, Read};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Config {
    #[structopt(short, long, parse(from_os_str))]
    notes_dir: std::path::PathBuf,
    #[structopt(short, long, parse(from_os_str))]
    downloads_dir: std::path::PathBuf,
}
impl Clone for Config {
    fn clone(&self) -> Self {
        Config {
            notes_dir: self.notes_dir.clone(),
            downloads_dir: self.downloads_dir.clone(),
        }
    }
}


fn main() {
    let config = Config::from_args();
    if !config.notes_dir.exists() {
        create_dir_all(&config.notes_dir).unwrap();
    }
    if !config.downloads_dir.exists() {
        create_dir_all(&config.downloads_dir).unwrap();
    }
    
    // Serve an echo service over HTTPS, with proper error handling.
    if let Err(e) = run_server(config) {
        eprintln!("FAILED: {}", e);
        std::process::exit(1);
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Note {
    id: String,
    content: String,
}


struct State {
    config: Config,
    notes_map: Arc<Mutex<HashMap<String, String>>>,
}

#[tokio::main]
async fn run_server<'a>(config: Config) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let port = 3000;
    let addr = format!("127.0.0.1:{}", port).parse()?;

    
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

    let initial_state = State {
        config: config.clone(),
        notes_map: Arc::new(Mutex::new(HashMap::new())),
    };
    let state = Arc::new(Mutex::new(initial_state));
    let service = make_service_fn(move |_| {
        let state = state.clone();
        async move {
            let state = state.clone();
            Ok::<_, Infallible>(service_fn(move |req| {
                let state = state.clone();
                handle(req, state)
            }))
        }
    });

    let server = Server::builder(acceptor).serve(service);

    // Run the future, keep going until an error occurs.
    println!("Starting to serve on https://{}.", addr);
    server.await?;
    Ok(())
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
    state: Arc<Mutex<State>>
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
            //todo: json content type header
            let dastate = state.lock().await;
            let mut map = dastate.notes_map.lock().await;

            let whole_body = hyper::body::to_bytes(req.into_body()).await.unwrap();
            let note: Note = serde_json::from_slice(&whole_body).unwrap();

            println!("{:?}", note);

            let file_path = get_path_for_note_id(&note.id, &dastate.config.notes_dir, &mut map);
            drop (map);

            let mut file = OpenOptions::new().create(true).write(true).truncate(true).open(&file_path).unwrap();
            file.write_all(note.content.as_bytes()).unwrap();

            let res: Response<Body> = Response::new("Note set".into());
            Ok(res)
        }
        (&Method::POST, "/download") => {
            let whole_body = hyper::body::to_bytes(req.into_body()).await.unwrap();
            let url = String::from_utf8(whole_body.to_vec()).unwrap();
        
            // Prepare the yt-dlp command as a string
            let yt_dlp_command = format!(
                "yt-dlp --cookies-from-browser edge --windows-filenames --embed-metadata {}",
                &url
            );
        
            // Prepare the entire command to open in a new Windows Terminal window
            let full_command = format!("start wt cmd /c \"{}\"", yt_dlp_command);
        
            let dir = &state.lock().await.config.downloads_dir;

            // Run the full command
            let output = std::process::Command::new("cmd")
                .current_dir(&dir)
                .args(&["/c", &full_command])
                .output()
                .expect("Failed to execute command");
        
            let res = if output.status.success() {
                println!("yt-dlp invoked successfully");
                Response::new("Downloaded".into())
            } else {
                eprintln!("yt-dlp failed: {}", String::from_utf8_lossy(&output.stderr));
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body("Download failed".into())
                    .unwrap()
            };
        
            Ok(res)
        }
        
        (&Method::GET, "/get_note") => {
            let dastate = state.lock().await;
            let mut map = dastate.notes_map.lock().await;
        
            // Decode the URI component and explicitly read from the query param named "id"
            let id = req.uri().query().unwrap().split("=").nth(1).unwrap();
            let decoded_id = percent_encoding::percent_decode_str(id).decode_utf8_lossy();
        
            println!("id: {}", decoded_id);
        
            let file_path = get_path_for_note_id(&decoded_id, &dastate.config.notes_dir, &mut map);
            drop(map);

            let mut content = String::new();
        
            if file_path.exists() {
                let mut file = OpenOptions::new().read(true).open(&file_path).unwrap();
                file.read_to_string(&mut content).unwrap();
            } else {
                println!("File not found: {:?} - creating empty note", file_path);
                let file = OpenOptions::new().create(true).append(true).open(&file_path).unwrap();
                drop(file);
            }
            // If file not found, content remains an empty string
            
            let note = Note {
                id: decoded_id.to_string(),
                content,
            };
            

            let res: Response<Body> = Response::new(serde_json::to_string(&note).unwrap().into());
            Ok(res)
        },             
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