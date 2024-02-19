use actix::prelude::*;
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client, RedisError};

pub struct RedisManager {
    con: MultiplexedConnection,
}

impl RedisManager {
    pub async fn new(connection_url: &str) -> Self {
        let client = Client::open(connection_url).unwrap();
        let con = client.get_multiplexed_async_connection().await.unwrap();
        RedisManager { con }
    }
}

impl Actor for RedisManager {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "Result<Option<String>, RedisError>")]
pub struct GetValue(pub String);

#[derive(Message)]
#[rtype(result = "Result<(), RedisError>")]
pub struct RemoveValue(pub String);

#[derive(Message)]
#[rtype(result = "Result<(), RedisError>")]
pub struct SetValue {
    pub key: String,
    pub value: String,
}

impl Handler<GetValue> for RedisManager {
    type Result = ResponseFuture<Result<Option<String>, RedisError>>;

    fn handle(&mut self, msg: GetValue, _: &mut Self::Context) -> Self::Result {
        // let Self { con } = self;
        let mut con = self.con.clone();
        let key = msg.0;
        Box::pin(async move {
            let value: Option<String> = con.get(&key).await?;
            Ok(value)
        })
    }
}

impl Handler<RemoveValue> for RedisManager {
    type Result = ResponseFuture<Result<(), RedisError>>;

    fn handle(&mut self, msg: RemoveValue, _: &mut Self::Context) -> Self::Result {
        let mut con = self.con.clone();
        let key = msg.0;
        Box::pin(async move {
            con.del(&key).await?;
            Ok(())
        })
    }
}

impl Handler<SetValue> for RedisManager {
    type Result = ResponseFuture<Result<(), RedisError>>;

    fn handle(&mut self, msg: SetValue, _: &mut Self::Context) -> Self::Result {
        let mut con = self.con.clone();
        let SetValue { key, value } = msg;
        Box::pin(async move {
            con.set(&key, &value).await?;
            Ok(())
        })
    }
}
