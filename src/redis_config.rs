use redis::{AsyncCommands, Client, RedisError};

pub struct RedisConfig {
    pub url: String,
}

impl RedisConfig {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct RedisData {
    client: Client,
}

impl RedisData {
    pub fn new(connection_url: &str) -> Result<Self, RedisError> {
        let client = Client::open(connection_url)?;
        Ok(Self { client })
    }

    pub async fn get_value(&self, key: &str) -> Result<Option<String>, redis::RedisError> {
        let mut con = self.client.get_async_connection().await?;
        let value: Option<String> = con.get(key).await?;
        Ok(value)
    }

    // remove key
    pub async fn remove_value(&self, key: &str) -> Result<(), redis::RedisError> {
        let mut con = self.client.get_async_connection().await?;
        con.del(key).await?;
        Ok(())
    }

    pub async fn set_value(&self, key: &str, value: &str) -> Result<(), redis::RedisError> {
        let mut con = self.client.get_async_connection().await?;
        con.set(key, value).await?;
        Ok(())
    }
}
