use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use std::convert::Infallible;
use systray::Application;
use tokio::sync::{Mutex, mpsc};
use std::sync::Arc;

async fn hello(_: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("Hello World")))
}

async fn run_server(mut shutdown_rx: mpsc::Receiver<()>) -> Result<(), Box<dyn std::error::Error + Send>> {
    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(hello))
    });

    let addr = ([127, 0, 0, 1], 3001).into();
    let server = Server::bind(&addr).serve(make_svc);

    let graceful = server.with_graceful_shutdown(async {
        println!("Waiting for stop signal");
        let _ = shutdown_rx.recv().await;
        println!("Stop signal received");
    });

    println!("Listening on http://{}", addr);

    if let Err(e) = graceful.await {
        return Err(Box::new(e));
    }
    Ok(())
}

async fn run_systray(shutdown_tx: Arc<Mutex<mpsc::Sender<()>>>) -> Result<(), systray::Error> {
    let mut app = Application::new()?;
    app.set_icon_from_file("icon.ico")?;
    app.add_menu_item("Quit", move |window| {
        let shutdown_tx_clone = Arc::clone(&shutdown_tx);
        tokio::spawn(async move {
            println!("Sending stop signal");
            let tx = shutdown_tx_clone.lock().await;
            _ = tx.send(()).await;
            println!("Stop signal sent");
        });
        window.quit();
        Ok::<_, systray::Error>(())
    })?;

    app.wait_for_message()?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
    let shutdown_tx = Arc::new(Mutex::new(shutdown_tx));

    let server_handle = tokio::spawn(run_server(shutdown_rx));
    let systray_handle = tokio::spawn(run_systray(shutdown_tx.clone()));

    let _ = tokio::try_join!(server_handle, systray_handle);
}
