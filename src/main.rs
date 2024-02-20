mod controller;
mod messages;
mod redis_manager;
mod ws;
mod ws_connection_manager;

use actix::{Actor, Message};
use actix_cors::Cors;
use actix_web::{get, middleware, web::Data, App, HttpServer};
use controller::websocket_service;
use redis::RedisError;
use redis_manager::RedisManager;
use ws_connection_manager::WsConnectionManager;

#[get("/health")]
async fn health_checkup() -> actix_web::Result<String> {
    Ok("{'message':'health ok'}".to_string())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    let redis_conn_url = format!("redis://:{}@{}", "poopmanroachman", "127.0.0.1");

    let redis_actor = RedisManager::new(&redis_conn_url).await.start();
    println!("Redis actor started!");

    let active_ws_manager = WsConnectionManager::new(redis_actor.clone()).start();

    println!("Server running on port 8080");

    HttpServer::new(move || {
        let cors = Cors::default().allow_any_origin().send_wildcard();

        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .app_data(Data::new(active_ws_manager.clone()))
            // .app_data(Data::new(redis_actor.clone()))
            .service(websocket_service)
            .service(health_checkup)
        // .service(set_key_value)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
