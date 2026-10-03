use std::sync::Arc;

use domain::repository::auth::AuthRepository;
use domain::repository::book::BookRepository;
use domain::repository::checkout::CheckoutRepository;
use domain::repository::health::HealthCheckRepository;
use domain::repository::user::UserRepository;
use infra::redis::RedisClient;
use infra::repository::auth::AuthRepositoryImpl;
use infra::repository::book::BookRepositoryImpl;
use infra::repository::checkout::CheckoutRepositoryImpl;
use infra::repository::user::UserRepositoryImpl;
use infra::{database::ConnectionPool, repository::health::HealthCheckRepositoryImpl};
use shared::config::AppConfig;

pub struct AppRegistryImpl {
    health_check_repository: Arc<dyn HealthCheckRepository>,
    book_repository: Arc<dyn BookRepository>,
    auth_repository: Arc<dyn AuthRepository>,
    user_repository: Arc<dyn UserRepository>,
    checkout_repository: Arc<dyn CheckoutRepository>,
}

impl AppRegistryImpl {
    pub fn new(
        pool: ConnectionPool,
        redis_client: Arc<RedisClient>,
        app_config: AppConfig,
    ) -> Self {
        let health_check_repository =
            Arc::new(HealthCheckRepositoryImpl::new(pool.clone()));
        let book_repository = Arc::new(BookRepositoryImpl::new(pool.clone()));
        let auth_repository = Arc::new(AuthRepositoryImpl::new(
            pool.clone(),
            redis_client.clone(),
            app_config.auth.ttl,
        ));
        let user_repository = Arc::new(UserRepositoryImpl::new(pool.clone()));
        let checkout_repository = Arc::new(CheckoutRepositoryImpl::new(pool.clone()));
        Self {
            health_check_repository,
            book_repository,
            auth_repository,
            user_repository,
            checkout_repository,
        }
    }
}

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait AppRegistry {
    fn health_check_repository(&self) -> Arc<dyn HealthCheckRepository>;
    fn book_repository(&self) -> Arc<dyn BookRepository>;
    fn auth_repository(&self) -> Arc<dyn AuthRepository>;
    fn user_repository(&self) -> Arc<dyn UserRepository>;
    fn checkout_repository(&self) -> Arc<dyn CheckoutRepository>;
}

impl AppRegistry for AppRegistryImpl {
    fn health_check_repository(&self) -> Arc<dyn HealthCheckRepository> {
        self.health_check_repository.clone()
    }

    fn book_repository(&self) -> Arc<dyn BookRepository> {
        self.book_repository.clone()
    }

    fn auth_repository(&self) -> Arc<dyn AuthRepository> {
        self.auth_repository.clone()
    }

    fn user_repository(&self) -> Arc<dyn UserRepository> {
        self.user_repository.clone()
    }

    fn checkout_repository(&self) -> Arc<dyn CheckoutRepository> {
        self.checkout_repository.clone()
    }
}

pub type SharedAppRegistry = Arc<dyn AppRegistry + Send + Sync + 'static>;
