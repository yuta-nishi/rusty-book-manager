pub mod model;

use self::model::{RedisKey, RedisValue};
use redis::{AsyncCommands, Client};
use shared::{config::RedisConfig, error::AppResult};

pub struct RedisClient {
    client: Client,
}

impl RedisClient {
    pub fn new(config: &RedisConfig) -> AppResult<Self> {
        let client = Client::open(format!("redis://{}:{}", config.host, config.port))?;
        Ok(Self { client })
    }

    pub async fn set_ex<T: RedisKey>(
        &self,
        key: &T,
        value: &T::Value,
        ttl: u64,
    ) -> AppResult<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let _: () = conn.set_ex(key.inner(), value.inner(), ttl).await?;
        Ok(())
    }

    pub async fn get<T: RedisKey>(&self, key: &T) -> AppResult<Option<T::Value>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let result: Option<String> = conn.get(key.inner()).await?;
        result.map(T::Value::try_from).transpose()
    }

    pub async fn delete<T: RedisKey>(&self, key: &T) -> AppResult<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let _: usize = conn.del(key.inner()).await?;
        Ok(())
    }

    pub async fn try_connect(&self) -> AppResult<()> {
        let _ = self.client.get_multiplexed_async_connection().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::error::AppError;

    #[derive(Debug, PartialEq, Eq)]
    struct TestContent {
        name: String,
    }

    struct TestContentKey(String);

    impl RedisKey for TestContentKey {
        type Value = TestContent;

        fn inner(&self) -> String {
            self.0.clone()
        }
    }

    impl TryFrom<String> for TestContent {
        type Error = AppError;

        fn try_from(value: String) -> Result<Self, Self::Error> {
            Ok(Self { name: value })
        }
    }

    impl RedisValue for TestContent {
        fn inner(&self) -> String {
            self.name.clone()
        }
    }

    #[tokio::test]
    async fn stores_and_deletes_a_value() -> anyhow::Result<()> {
        let config = RedisConfig {
            host: std::env::var("REDIS_HOST")?,
            port: std::env::var("REDIS_PORT")?.parse()?,
        };
        let client = RedisClient::new(&config)?;
        let key = TestContentKey("test:key".to_string());
        let content = TestContent {
            name: "value".to_string(),
        };

        assert!(client.get(&key).await?.is_none());

        client.set_ex(&key, &content, 1000).await?;
        assert_eq!(client.get(&key).await?, Some(content));

        client.delete(&key).await?;
        assert!(client.get(&key).await?.is_none());

        Ok(())
    }
}
