use crate::messages::{
    BroadcastEvent, BroadcastEventBody, Connect, Disconnect, StreamEvent, StreamEventBody,
    WsMessage,
};
use crate::redis_config::RedisData;
use crate::ws_connection_manager::WsConnectionManager;
use actix::{fut, ActorContext, ActorFuture, ContextFutureSpawner, WrapFuture};
use actix::{Actor, Addr, Running, StreamHandler};
use actix::{AsyncContext, Handler};
use actix_web_actors::ws;
use actix_web_actors::ws::Message::Text;
use std::time::{Duration, Instant};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct WsConnection {
    connectionManager: Addr<WsConnectionManager>,
    hb: Instant,
    id: String,
    client_id: String,
}

impl WsConnection {
    pub fn new(
        user_id: String,
        client_id: String,
        connectionManager: Addr<WsConnectionManager>,
    ) -> Self {
        Self {
            id: user_id,
            client_id,
            hb: Instant::now(),
            connectionManager,
        }
    }
}

impl Actor for WsConnection {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);

        let addr = ctx.address();
        actix::ActorFutureExt::then(
            self.connectionManager
                .send(Connect {
                    addr: addr.recipient(),
                    client_id: self.client_id.clone(),
                    user_id: self.id.clone(),
                })
                .into_actor(self),
            |res, _, ctx| {
                match res {
                    Ok(_res) => (),
                    _ => ctx.stop(),
                }
                fut::ready(())
            },
        )
        .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.connectionManager.do_send(Disconnect {
            user_id: self.id.clone(),
            client_id: self.client_id.clone(),
        });
        Running::Stop
    }
}

impl WsConnection {
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                println!("Disconnecting failed heartbeat");
                ctx.stop();
                return;
            }

            ctx.ping(b"hi");
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsConnection {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Continuation(_)) => {
                ctx.stop();
            }
            Ok(ws::Message::Nop) => (),
            Ok(Text(s)) => {
                let msg: WsMessage = serde_json::from_str(&s).unwrap();

                if msg.message_type == "broadcast_event" {
                    let data: BroadcastEventBody = serde_json::from_str(&msg.data).unwrap();
                    self.connectionManager.do_send(BroadcastEvent {
                        user_id: self.id.clone(),
                        client_id: self.client_id.clone(),
                        data,
                    });
                } else if msg.message_type == "stream_event" {
                    let data: StreamEventBody = serde_json::from_str(&msg.data).unwrap();
                    self.connectionManager.do_send(StreamEvent {
                        user_id: self.id.clone(),
                        client_id: self.client_id.clone(),
                        data,
                    });
                } else if msg.message_type == "game_data" {
                    // self.connectionManager.do_send(msg);
                }
                //      self.connectionManager.do_send(StreamEvent {
                //     user_id: self.id.clone(),
                //     msg: s,
                //     client_id: self.client_id.clone(),
                // }),
            }
            Err(e) => std::panic::panic_any(e),
        }
    }
}

impl Handler<WsMessage> for WsConnection {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(serde_json::to_string(&msg).unwrap());
    }
}
