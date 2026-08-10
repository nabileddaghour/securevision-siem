#![allow(dead_code)]

use redis::AsyncCommands;

#[derive(Clone)]
pub struct RedisCache {
    client: redis::Client,
}

impl RedisCache {
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self { client })
    }

    pub async fn set_key(&self, key: &str, value: &str, ttl_seconds: u64) -> Result<(), redis::RedisError> {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        let _: () = con.set_ex(key, value, ttl_seconds).await?;
        Ok(())
    }

    pub async fn get_key(&self, key: &str) -> Result<Option<String>, redis::RedisError> {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        let val: Option<String> = con.get(key).await?;
        Ok(val)
    }
}
