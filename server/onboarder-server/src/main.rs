use chrono::{Datelike, Local};
use hyper::server::conn::AddrIncoming;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use hyper::{Method, StatusCode};
use hyper_rustls::TlsAcceptor;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::env;
use std::fs::{create_dir_all, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Arc;
use structopt::StructOpt;
use tokio::sync::Mutex;
use tracing::level_filters::LevelFilter;
use tracing::{debug, error, info, trace};
use tracing_subscriber::EnvFilter;

#[derive(StructOpt)]
struct Config {
    #[structopt(short, long, parse(from_os_str))]
    notes_dir: std::path::PathBuf,
    #[structopt(short, long, parse(from_os_str))]
    downloads_dir: std::path::PathBuf,
    #[structopt(short, long, parse(from_os_str))]
    search_dirs: Vec<std::path::PathBuf>,
    #[structopt(short, long, parse(try_from_str))]
    port: usize,
}
impl Clone for Config {
    fn clone(&self) -> Self {
        Config {
            notes_dir: self.notes_dir.clone(),
            downloads_dir: self.downloads_dir.clone(),
            search_dirs: self.search_dirs.clone(),
            port: self.port.clone(),
        }
    }
}

fn main() {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy()
                .add_directive(
                    "
                        onboarder_server=debug
                    "
                    .lines()
                    .map(|line| line.trim())
                    .filter(|line| !line.starts_with("//"))
                    .filter(|line| !line.is_empty())
                    .join(",")
                    .trim()
                    .parse()
                    .unwrap(),
                ),
        )
        .init();

    let config = Config::from_args();
    if !config.notes_dir.exists() {
        create_dir_all(&config.notes_dir).unwrap();
    }
    if !config.downloads_dir.exists() {
        create_dir_all(&config.downloads_dir).unwrap();
    }

    // Serve an echo service over HTTPS, with proper error handling.
    if let Err(e) = run_server(config) {
        error!("FAILED: {}", e);
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
    let addr = format!("127.0.0.1:{}", config.port).parse()?;
    info!("Attempting to listen on https://{}", addr);

    // Ensure our certs are available before binding
    let certs = load_certs("onboarder+1.pem")?;
    let key = load_private_key("onboarder+1-key.pem")?;

    // Create a TCP listener via tokio.
    let mut incoming = None;
    for _ in 0..25 {
        match AddrIncoming::bind(&addr) {
            Ok(incoming_) => {
                incoming = Some(incoming_);
                break;
            }
            Err(e) => {
                error!("Failed to bind to port: {}", e);
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }

    let Some(incoming) = incoming else {
        error!("Failed to bind to port after 25 attempts");
        std::process::exit(1);
    };
    info!("Successfully bound port");

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
    info!("Starting to serve on https://{}.", addr);
    server.await?;
    Ok(())
}

fn get_path_for_note_id(
    id: &str,
    notes_dir: &PathBuf,
    notes_map: &mut HashMap<String, String>,
) -> std::io::Result<PathBuf> {
    if let Some(path) = notes_map.get(id) {
        return Ok(PathBuf::from(path));
    }

    let invalid_chars: Vec<char> = vec!['<', '>', ':', '"', '/', '\\', '|', '?', '*', '\n'];
    let sanitized_id = id
        .chars()
        .map(|c| if invalid_chars.contains(&c) { '_' } else { c })
        .collect::<String>();

    let dated_dir = get_dated_dir(notes_dir)?;
    let file_path = dated_dir.join(format!("{}.txt", &sanitized_id));

    notes_map.insert(id.to_string(), file_path.to_str().unwrap().to_string());

    Ok(file_path)
}

async fn search(text: &str, dir: &PathBuf) -> Result<Vec<String>, String> {
    let shell_code = format!(
        "Get-ChildItem -Recurse {} | ForEach-Object {{ $_.FullName}} | rg --ignore-case --fixed-strings --regexp '{}'",
        dir.to_str().unwrap(),
        text
    );
    debug!("shell_code: {}", shell_code);
    let output = tokio::process::Command::new("pwsh")
        .arg("-c")
        .arg(shell_code)
        .output()
        .await
        .expect("Failed to execute command");

    if output.status.success() {
        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let results: Vec<String> = stdout_str
            .lines()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect();
        Ok(results)
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn get_dated_dir(parent_dir: &PathBuf) -> std::io::Result<PathBuf> {
    let now = Local::now();
    let dated_dir = parent_dir.join(format!(
        "{}/{:02}/{:02}",
        now.year(),
        now.month(),
        now.day()
    ));
    create_dir_all(&dated_dir)?;
    Ok(dated_dir)
}

async fn get_ytdlp_filename(url: &str) -> Result<String, String> {
    let output = tokio::process::Command::new("yt-dlp")
        .arg("--encoding")
        .arg("utf-8")
        .arg("--print")
        .arg("filename")
        // .arg("--cookies-from-browser")
        // .arg("edge")
        .arg("--windows-filenames")
        .arg("--embed-metadata")
        .arg(url)
        .output()
        .await
        .expect("Failed to execute command");

    if output.status.success() {
        let fname = String::from_utf8_lossy(&output.stdout);
        let fname_trimmed = fname.trim_end_matches('\n');
        Ok(fname_trimmed.to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
async fn get_ytdlp_audio_filename(url: &str) -> Result<String, String> {
    let output = tokio::process::Command::new("yt-dlp")
        .arg("--encoding")
        .arg("utf-8")
        .arg("--print")
        .arg("filename")
        // .arg("--cookies-from-browser")
        // .arg("edge")
        .arg("-f")
        .arg("bestaudio")
        .arg("--extract-audio")
        .arg("--windows-filenames")
        .arg("--embed-metadata")
        .arg(url)
        .output()
        .await
        .expect("Failed to execute command");

    if output.status.success() {
        let fname = String::from_utf8_lossy(&output.stdout);
        let fname_trimmed = fname.trim_end_matches('\n');
        Ok(fname_trimmed.to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

async fn handle(
    req: Request<Body>,
    state: Arc<Mutex<State>>,
) -> Result<Response<Body>, Infallible> {
    info!("{} {}", req.method(), req.uri().path());
    match req.uri().query() {
        Some(query) => info!("query: {}", query),
        None => {},
    }
    let mut res = match (req.method(), req.uri().path()) {
        (&Method::OPTIONS, _) => {
            let res: Response<Body> = Response::new("".into());
            Ok::<Response<Body>, Infallible>(res)
        }
        (&Method::GET, "/healthcheck") => {
            let mut res = Response::default();
            *res.status_mut() = StatusCode::OK;
            Ok(res)
        }
        (&Method::POST, "/set_note") => {
            //todo: json content type header
            let dastate = state.lock().await;
            let mut map = dastate.notes_map.lock().await;

            let whole_body = hyper::body::to_bytes(req.into_body()).await.unwrap();
            let note: Note = serde_json::from_slice(&whole_body).unwrap();

            trace!("{:?}", note);

            let file_path =
                match get_path_for_note_id(&note.id, &dastate.config.notes_dir, &mut map) {
                    Ok(it) => it,
                    Err(err) => {
                        error!("Error getting path for note id: {}", err);
                        return Ok(Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body("Error getting path for note id".into())
                            .unwrap());
                    }
                };
            drop(map);

            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&file_path)
                .unwrap();
            let bytes = note.content.as_bytes();
            info!("Writing {} bytes to \"{}\"", bytes.len(), file_path.display());
            file.write_all(bytes).unwrap();

            let res: Response<Body> = Response::new("Note set".into());
            Ok(res)
        }
        (&Method::GET, "/exists") => {
            let query_map = url::form_urlencoded::parse(req.uri().query().unwrap_or("").as_bytes())
                .into_owned()
                .collect::<HashMap<String, String>>();

            if let Some(search_param) = query_map.get("search") {
                let decoded_search =
                    percent_encoding::percent_decode_str(search_param).decode_utf8_lossy();

                let dirs = &state.lock().await.config.search_dirs;
                let mut total_results = Vec::new();

                for dir in dirs {
                    match search(&decoded_search, dir).await {
                        Ok(results) => {
                            total_results.extend(results);
                        }
                        Err(err) => {
                            error!("Error in search: {}", err);
                            info!("Sending 'not found' result due to error");
                        }
                    }
                }

                let res = if total_results.is_empty() {
                    Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body("No results found".into())
                        .unwrap()
                } else {
                    Response::new(format!("Results: {:?}", total_results).into())
                };

                Ok(res)
            } else {
                Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body("Missing search parameter".into())
                    .unwrap())
            }
        }

        (&Method::POST, "/download") => {
            let whole_body = hyper::body::to_bytes(req.into_body()).await.unwrap();
            let url = String::from_utf8(whole_body.to_vec()).unwrap();

            let dir = &state.lock().await.config.downloads_dir;
            let dated_dir = match get_dated_dir(dir) {
                Ok(it) => it,
                Err(err) => {
                    error!("Error getting dated dir: {}", err);
                    return Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body("Error getting dated dir".into())
                        .unwrap());
                }
            };

            let filename = get_ytdlp_filename(&url)
                .await
                .expect("Failed to get filename");

            // Run the full command
            let output = std::process::Command::new("pwsh")
                .current_dir(&dated_dir)
                .arg("-NoProfile")
                .arg("-WorkingDirectory")
                .arg("$(Get-Location)")
                .arg("-c")
                // .arg(format!("wt pwsh.exe -NoProfile -WorkingDirectory $(Get-Location) -c 'yt-dlp --cookies-from-browser edge --windows-filenames --embed-metadata \"{}\" && Write-Host \"\"press any key to close\"\" && $Host.UI.RawUI.ReadKey(\"\"NoEcho,IncludeKeyDown\"\")'", url))
                .arg(format!("wt pwsh.exe -NoProfile -WorkingDirectory $(Get-Location) -c 'yt-dlp --windows-filenames --write-subs --write-auto-subs --embed-metadata \"{}\" && Write-Host \"\"press any key to close\"\" && $Host.UI.RawUI.ReadKey(\"\"NoEcho,IncludeKeyDown\"\")'", url))
                .output()
                .expect("Failed to execute command");

            let res = if output.status.success() {
                info!("Subprocess invoked successfully");
                Response::new(filename.into())
            } else {
                error!(
                    "Subprocess failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body("download failed".into())
                    .unwrap()
            };

            Ok(res)
        }

        (&Method::POST, "/download_audio") => {
            let whole_body = hyper::body::to_bytes(req.into_body()).await.unwrap();
            let url = String::from_utf8(whole_body.to_vec()).unwrap();

            let dir = &state.lock().await.config.downloads_dir;
            let dated_dir = match get_dated_dir(dir) {
                Ok(it) => it,
                Err(err) => {
                    error!("Error getting dated dir: {}", err);
                    return Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body("Error getting dated dir".into())
                        .unwrap());
                }
            };

            let filename = get_ytdlp_audio_filename(&url)
                .await
                .expect("Failed to get filename");

            // Run the full command
            let output = std::process::Command::new("pwsh")
                .current_dir(&dated_dir)
                .arg("-NoProfile")
                .arg("-WorkingDirectory")
                .arg("$(Get-Location)")
                .arg("-c")
                // .arg(format!("wt pwsh.exe -NoProfile -WorkingDirectory $(Get-Location) -c 'yt-dlp --cookies-from-browser edge --windows-filenames --embed-metadata \"{}\" && Write-Host \"\"press any key to close\"\" && $Host.UI.RawUI.ReadKey(\"\"NoEcho,IncludeKeyDown\"\")'", url))
                .arg(format!("wt pwsh.exe -NoProfile -WorkingDirectory $(Get-Location) -c 'yt-dlp \"{}\" -f bestaudio --extract-audio --windows-filenames --embed-metadata  && Write-Host \"\"press any key to close\"\" && $Host.UI.RawUI.ReadKey(\"\"NoEcho,IncludeKeyDown\"\")'", url))
                .output()
                .expect("Failed to execute command");

            let res = if output.status.success() {
                info!("Subprocess invoked successfully");
                Response::new(filename.into())
            } else {
                error!(
                    "Subprocess failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body("download failed".into())
                    .unwrap()
            };

            Ok(res)
        }

        (&Method::POST, "/open_videos_folder") => {
            let whole_body = hyper::body::to_bytes(req.into_body()).await.unwrap();
            let url = String::from_utf8(whole_body.to_vec()).unwrap();
            info!("Opening folder for {}", url);

            let dir = &state.lock().await.config.downloads_dir;
            let dated_dir = match get_dated_dir(dir) {
                Ok(it) => it,
                Err(err) => {
                    error!("Error getting dated dir: {}", err);
                    return Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body("Error getting dated dir".into())
                        .unwrap());
                }
            };

            let absolute_path = env::current_dir()
                .unwrap()
                .join(&dated_dir)
                .canonicalize()
                .unwrap();

            let output = std::process::Command::new("explorer.exe")
                .current_dir(&dated_dir)
                .arg(absolute_path.clone())
                .output()
                .expect("Failed to execute command");

            let res = if output.status.success() {
                info!("Subprocess invoked successfully");
                // Convert the absolute path to a String
                let path_str = absolute_path.to_str().unwrap().to_string();
                // Create the response with the string body
                Response::new(Body::from(path_str))
            } else {
                error!(
                    "Subprocess failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body("download failed".into())
                    .unwrap()
            };

            Ok(res)
        }

        (&Method::POST, "/open_notes_folder") => {
            let whole_body = hyper::body::to_bytes(req.into_body()).await.unwrap();
            let url = String::from_utf8(whole_body.to_vec()).unwrap();
            info!("Opening folder for {}", url);

            let dir = &state.lock().await.config.notes_dir;
            let dated_dir = match get_dated_dir(dir) {
                Ok(it) => it,
                Err(err) => {
                    error!("Error getting dated dir: {}", err);
                    return Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body("Error getting dated dir".into())
                        .unwrap());
                }
            };

            let absolute_path = env::current_dir()
                .unwrap()
                .join(&dated_dir)
                .canonicalize()
                .unwrap();

            let output = std::process::Command::new("explorer.exe")
                .current_dir(&dated_dir)
                .arg(absolute_path.clone())
                .output()
                .expect("Failed to execute command");

            let res = if output.status.success() {
                info!("Subprocess invoked successfully");
                // Convert the absolute path to a String
                let path_str = absolute_path.to_str().unwrap().to_string();
                // Create the response with the string body
                Response::new(Body::from(path_str))
            } else {
                error!(
                    "Subprocess failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body("download failed".into())
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

            info!("id: {}", decoded_id);

            let file_path =
                match get_path_for_note_id(&decoded_id, &dastate.config.notes_dir, &mut map) {
                    Ok(it) => it,
                    Err(err) => {
                        error!("Error getting path for note id: {}", err);
                        return Ok(Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body("Error getting path for note id".into())
                            .unwrap());
                    }
                };
            drop(map);

            let mut content = String::new();

            if file_path.exists() {
                let mut file = OpenOptions::new().read(true).open(&file_path).unwrap();
                file.read_to_string(&mut content).unwrap();
            } else {
                info!("File not found: {:?} - creating empty note", file_path);
                let file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&file_path)
                    .unwrap();
                drop(file);
            }
            // If file not found, content remains an empty string

            let note = Note {
                id: decoded_id.to_string(),
                content,
            };

            let res: Response<Body> = Response::new(serde_json::to_string(&note).unwrap().into());
            Ok(res)
        }
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }?;
    let headers = res.headers_mut();
    headers.insert(
        "Access-Control-Allow-Origin",
        "https://www.youtube.com".parse().unwrap(),
    );
    headers.insert(
        "Access-Control-Allow-Methods",
        "GET, POST, OPTIONS".parse().unwrap(),
    );
    headers.insert(
        "Access-Control-Allow-Headers",
        "Content-Type".parse().unwrap(),
    );
    let status = res.status();
    info!("Response: {}", status);

    Ok(res)
}

// Load public certificate from file.
fn load_certs(filename: &str) -> std::io::Result<Vec<rustls::Certificate>> {
    // Open certificate file.
    let certfile = std::fs::File::open(filename).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!(
                "failed to open {} (did you run gen-certs.ps1?): {}",
                filename, e
            ),
        )
    })?;
    let mut reader = std::io::BufReader::new(certfile);

    // Load and return certificate.
    let certs = rustls_pemfile::certs(&mut reader).map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::Other, "failed to load certificate")
    })?;
    Ok(certs.into_iter().map(rustls::Certificate).collect())
}

// Load private key from file.
fn load_private_key(filename: &str) -> std::io::Result<rustls::PrivateKey> {
    // Open keyfile.
    let keyfile = std::fs::File::open(filename).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!(
                "failed to open {} (did you run gen-certs.ps1?): {}",
                filename, e
            ),
        )
    })?;
    let mut reader = std::io::BufReader::new(keyfile);

    // Load and return a single private key.
    let keys = rustls_pemfile::pkcs8_private_keys(&mut reader).map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::Other, "failed to load private key")
    })?;
    if keys.len() != 1 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("expected a single private key, got {} instead", keys.len()),
        ));
    }

    Ok(rustls::PrivateKey(keys[0].clone()))
}
