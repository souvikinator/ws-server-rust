use crate::{
    messages::{BroadcastEvent, BroadcastState, Connect, Disconnect, StreamEvent, WsMessage},
    redis_config::RedisData,
};
use actix::{
    prelude::{Actor, Context, Handler, Recipient},
    AsyncContext, ResponseFuture, WrapFuture,
};
use std::collections::HashMap;

type Socket = Recipient<WsMessage>;

pub struct WsConnectionManager {
    sessions: HashMap<String, Socket>,
    redis_broadcast_manager: RedisData,
}

impl WsConnectionManager {
    pub fn new(redis_broadcast_manager: RedisData) -> WsConnectionManager {
        WsConnectionManager {
            sessions: HashMap::new(),
            redis_broadcast_manager,
        }
    }

    pub async fn remove_stream(&self, stream_id: &str) {
        self.redis_broadcast_manager
            .remove_value(stream_id)
            .await
            .unwrap();
    }

    pub async fn get_stream_viewers(&self, stream_id: &str) -> Vec<BroadcastState> {
        let raw_stream_viewers = self
            .redis_broadcast_manager
            .get_value(stream_id)
            .await
            .unwrap();

        let stream_viewers: Vec<BroadcastState> =
            serde_json::from_str(raw_stream_viewers.as_ref().unwrap().as_str()).unwrap();

        stream_viewers
    }

    pub async fn set_stream_viewers(&self, stream_id: &str, viewers: Vec<BroadcastState>) {
        let viewers_json = serde_json::to_string(&viewers).unwrap();
        self.redis_broadcast_manager
            .set_value(stream_id, &viewers_json)
            .await
            .unwrap();
    }

    pub fn send_message(&self, message: &str, message_type: String, id_to: &String) {
        if let Some(socket_recipient) = self.sessions.get(id_to) {
            let _ = socket_recipient.do_send(WsMessage {
                message_type,
                data: message.to_owned(),
            });
        } else {
            println!("attempting to send message but couldn't find user id.");
        }
    }

    pub async fn broadcast_message_to_stream(
        &self,
        message: &str,
        message_type: String,
        stream_id: &str,
    ) {
        let stream_viewers = self.get_stream_viewers(stream_id).await;

        // iterate through stream_viewers and send message to each
        if let viewers = stream_viewers {
            for viewer in viewers {
                self.send_message(message, message_type.clone(), &viewer.viewer_id);
            }
        }
    }

    pub async fn remove_from_stream(&self, viewer_id: &str, stream_id: &str) {
        let mut stream_viewers = self.get_stream_viewers(stream_id).await;

        // remove viewer_id from stream_viewers
        stream_viewers.retain(|viewer| viewer.viewer_id != viewer_id);

        // set stream_viewers to redis
        self.set_stream_viewers(stream_id, stream_viewers).await;

        // TODO: broadcast to all viewers that viewer_id has left the stream
    }
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

impl Handler<BroadcastEvent> for WsConnectionManager {
    type Result = ResponseFuture<Result<(), ()>>;

    fn handle(&mut self, msg: BroadcastEvent, ctx: &mut Context<Self>) -> Self::Result {
        let key = format!("player:{}:{}", msg.user_id, msg.client_id);

        if msg.data.action == "broadcast_start" {
            let fut = async move {
                // make an entry in redis with stream_id as key and value as empty array
                self.set_stream_viewers(&key, vec![]).await;
                // check for any previous entry, if any then broadcast end stream to everyone
                println!("STARTING STREAM: {}", key);
                Ok(())
            };
            Box::pin(fut)
        } else {
            // Handle other cases or return an error result
            Box::pin(async { Err(()) })
        }
        // } else if msg.data.action == "broadcast_end" {
        //     let fut = async move {
        //         // remove entry from redis and broadcast end stream to everyone
        //         self.remove_stream(&key).await;
        //         println!("STOPPING STREAM: {}", key);
        //         Ok(())
        //     };
        //     Box::pin(fut);
        // }
        // broadcast
        // self.rooms
        //     .get(&msg.room_id)
        //     .unwrap()
        //     .iter()
        //     .for_each(|client| self.send_message(&msg.msg, client));
    }
}

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
            self.remove_from_stream(&key, &stream_id);
            println!("LEAVING STREAM: {}", msg.user_id);
        } else {
            println!("UNKNOWN ACTION: {}", msg.data.action);
        }
    }
}
