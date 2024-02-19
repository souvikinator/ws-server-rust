use crate::{
    messages::{BroadcastEvent, Connect, Disconnect, StreamEvent, WsMessage},
    redis_manager::{GetValue, RedisManager, RemoveValue, SetValue},
};
use actix::{
    dev::ContextFutureSpawner,
    fut,
    prelude::{Actor, Context, Handler, Recipient},
    ActorFutureExt, Addr, AsyncContext, WrapFuture,
};
use redis::RedisError;
use serde::{Deserialize, Serialize};

use std::{collections::HashMap, fmt::format};

type Socket = Recipient<WsMessage>;

#[derive(Serialize, Deserialize)]
pub struct StreamViewer {
    pub viewer_id: String,
}

pub struct WsConnectionManager {
    sessions: HashMap<String, Socket>,
    redis_actor: Addr<RedisManager>,
}

impl WsConnectionManager {
    pub fn new(redis_actor: Addr<RedisManager>) -> WsConnectionManager {
        WsConnectionManager {
            sessions: HashMap::new(),
            redis_actor,
        }
    }

    pub fn send_message(&mut self, message: &str, message_type: String, id_to: &String) {
        if let Some(socket_recipient) = self.sessions.get(id_to) {
            let _ = socket_recipient.do_send(WsMessage {
                message_type,
                data: message.to_owned(),
            });
        } else {
            println!("attempting to send message but couldn't find user id.");
        }
    }

    //     pub async fn remove_stream(&mut self, stream_id: &str) {
    //         self.redis_broadcast_manager
    //             .remove_value(stream_id)
    //             .await
    //             .unwrap();
    //     }

    //     pub async fn get_stream_viewers(&mut self, stream_id: &str) -> Vec<BroadcastState> {
    //         let raw_stream_viewers = self
    //             .redis_broadcast_manager
    //             .get_value(stream_id)
    //             .await
    //             .unwrap();

    //         let stream_viewers: Vec<BroadcastState> =
    //             serde_json::from_str(raw_stream_viewers.as_ref().unwrap().as_str()).unwrap();

    //         stream_viewers
    //     }

    //     pub async fn set_stream_viewers(&mut self, stream_id: &str, viewers: Vec<BroadcastState>) {
    //         let viewers_json = serde_json::to_string(&viewers).unwrap();
    //         self.redis_broadcast_manager
    //             .set_value(stream_id, &viewers_json)
    //             .await
    //             .unwrap();
    //     }

    pub fn broadcast_message_to_viewers(
        &mut self,
        message: &str,
        message_type: String,
        stream_viewers: &Vec<StreamViewer>,
    ) {
        // iterate through stream_viewers and send message to each
        if let viewers = stream_viewers {
            for viewer in viewers {
                self.send_message(message, message_type.clone(), &viewer.viewer_id);
            }
        }
    }

    //     pub async fn remove_from_stream(&mut self, viewer_id: &str, stream_id: &str) {
    //         let mut stream_viewers = self.get_stream_viewers(stream_id).await;

    //         // remove viewer_id from stream_viewers
    //         stream_viewers.retain(|viewer| viewer.viewer_id != viewer_id);

    //         // set stream_viewers to redis
    //         self.set_stream_viewers(stream_id, stream_viewers).await;

    //         // TODO: broadcast to all viewers that viewer_id has left the stream
    //     }
}

impl Actor for WsConnectionManager {
    type Context = Context<Self>;
}

impl Handler<Disconnect> for WsConnectionManager {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        let key = format!("{}:{}", msg.user_id, msg.client_id);
        if self.sessions.remove(&key).is_some() {
            // send message to all other users in the room?
            println!("DISCONNECTED: {}", key);
        }
    }
}

impl Handler<Connect> for WsConnectionManager {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        let key = format!("{}:{}", msg.user_id, msg.client_id);
        self.sessions.insert(key.clone(), msg.addr);

        // self.send_message(&format!("your id is {}", key.clone()), &key);

        println!("ACTIVE SESSION COUNT: {}", self.sessions.len());
    }
}

// handle ws message related to start and end game broadcast
impl Handler<BroadcastEvent> for WsConnectionManager {
    type Result = ();

    fn handle(&mut self, msg: BroadcastEvent, ctx: &mut Context<Self>) -> Self::Result {
        let player_id = format!("{}:{}", msg.user_id, msg.client_id);
        let key = format!("player:{}", player_id);

        if msg.data.action == "broadcast_start" {
            let mut viewers: Vec<StreamViewer> = Vec::new();
            let new_viewer = StreamViewer {
                viewer_id: player_id,
            };

            // add player as viewer
            viewers.push(new_viewer);

            self.redis_actor.do_send(SetValue {
                key: key.clone(),
                value: serde_json::to_string(&viewers).unwrap(),
            });
        } else if msg.data.action == "broadcast_end" {
            let future = self
                .redis_actor
                .send(GetValue(key.clone()))
                .into_actor(self)
                .then(move |res, act, _| {
                    match res {
                        Ok(res) => {
                            let redis_response = res.unwrap().unwrap();
                            let viewers: Vec<StreamViewer> =
                                serde_json::from_str(redis_response.as_str()).unwrap();

                            act.broadcast_message_to_viewers(
                                "broadcast_end",
                                "broadcast_event".to_string(),
                                &viewers,
                            );

                            act.redis_actor.do_send(RemoveValue(key.clone()));
                        }
                        Err(err) => {
                            println!("Error getting value from Redis: {:?}", err);
                        }
                    }
                    fut::ready(())
                });

            // Spawn the future
            ctx.spawn(future);
        } else {
            println!("UNKNOWN ACTION: {}", msg.data.action);
        }
    }
}

// handle ws message related to joining and exiting stream
impl Handler<StreamEvent> for WsConnectionManager {
    type Result = ();

    fn handle(&mut self, msg: StreamEvent, _ctx: &mut Context<Self>) -> Self::Result {
        let key = format!("{}:{}", msg.user_id, msg.client_id);
        let stream_id = msg.data.stream_id.clone();

        if msg.data.action == "stream_join" {
            // TODO: check if stream active on redis? no then return streamEventResponse
            // if yes then add user to the room and request for send_game_state
            // if stream is active then add user to the stream_id in redis
            println!("JOINING STREAM: {}", msg.user_id);
        } else if msg.data.action == "stream_leave" {
            // remove user from the stream_id in redis
            // self.remove_from_stream(&key, &stream_id);
            println!("LEAVING STREAM: {}", msg.user_id);
        } else {
            println!("UNKNOWN ACTION: {}", msg.data.action);
        }
    }
}
