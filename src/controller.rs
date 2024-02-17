use crate::{ws::WsConnection, ws_connection_manager::WsConnectionManager};
use actix::Addr;
use actix_web::{get, web::Data, web::Path, web::Payload, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use uuid::Uuid;

#[get("/connect/{user_id}/{client_id}")]
pub async fn websocket_service(
    req: HttpRequest,
    stream: Payload,
    srv: Data<Addr<WsConnectionManager>>,
) -> Result<HttpResponse, Error> {
    let user_id: String = req.match_info().get("user_id").unwrap().to_string();
    let client_id: String = req.match_info().get("client_id").unwrap().to_string();

    let ws = WsConnection::new(user_id, client_id, srv.get_ref().clone());

    let resp = ws::start(ws, &req, stream)?;
    Ok(resp)
}
