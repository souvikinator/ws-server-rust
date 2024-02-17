mod controller;
mod messages;
mod redis_config;
mod ws;
mod ws_connection_manager;

use actix::Actor;
use actix_cors::Cors;
use actix_web::{get, middleware, web::Data, App, HttpRequest, HttpServer};
use controller::websocket_service;
use redis::AsyncCommands;
use redis_config::{RedisConfig, RedisData};
use ws_connection_manager::WsConnectionManager;

// create a endpoint that takes two values as key and value and stores it in redis
// #[get("/set/{key}/{value}")]
// async fn set_key_value(
//     redis_data: actix_web::web::Data<RedisData>,
//     req: HttpRequest,
// ) -> actix_web::Result<String> {
//     let key = req.match_info().get("key").unwrap();
//     let value = req.match_info().get("value").unwrap();
//     redis_data.set_value(&key, &value).await.unwrap();
//     Ok(format!("Set key: {} to value: {}", key, value))
// }

#[get("/health")]
async fn health_checkup() -> actix_web::Result<String> {
    Ok("{'message':'health ok'}".to_string())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    let redis_conn_url = format!("redis://:{}@{}", "poopmanroachman", "127.0.0.1");

    let redis_config = RedisConfig::new(&redis_conn_url);
    let redis_data = RedisData::new(&redis_config.url);
    // handle redisError
    let redis_data = match redis_data {
        Ok(data) => {
            println!("Connected to redis");
            data
        }
        Err(e) => {
            panic!("Error connecting to redis: {}", e);
        }
    };

    let active_ws_manager = WsConnectionManager::new(redis_data).start();

    println!("Server running on port 8080");

    HttpServer::new(move || {
        let cors = Cors::default().allow_any_origin().send_wildcard();

        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .app_data(Data::new(active_ws_manager.clone()))
            .service(websocket_service)
            .service(health_checkup)
        // .service(set_key_value)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
