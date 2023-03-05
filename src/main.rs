use crate::config::{AppConfig, ServerConfig};
use actix_web::{App, HttpServer};
use std::io;

mod cli;
mod config;
mod db;
mod route;
mod util;

#[actix_web::main]
async fn main() -> io::Result<()> {
    let matches = cli::get_matches();
    let app_config = AppConfig::from(&matches);
    let server_config = ServerConfig::from(&matches);
    let pool = server_config.open_database_pool().await.unwrap();
    let port = app_config.port;

    let mut server = HttpServer::new(move || {
        App::new()
            .app_data(app_config.clone())
            .app_data(pool.clone())
            .service(route::upload)
            .service(route::delete)
    });

    if let Some(port) = &port {
        server = server.bind(format!("127.0.0.1:{port}"))?;
        println!("Server running at: http://localhost:{port}/");
    }

    #[cfg(unix)]
    {
        if let Some(unix) = &server_config.unix {
            server = server.bind_uds(unix)?;
            println!("Server socket running at: {}", unix.display());
        }
    }

    server.keep_alive(None).run().await
}
