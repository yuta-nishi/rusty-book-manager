use async_trait::async_trait;

#[cfg_attr(feature = "mock", mockall::automock)]
#[async_trait]
pub trait HealthCheckRepository: Send + Sync {
    async fn check_db(&self) -> bool;
}
