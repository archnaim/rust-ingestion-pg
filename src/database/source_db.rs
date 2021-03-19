use std::env;
use sqlx::any::AnyPoolOptions;

pub async fn create_pool() -> Result<sqlx::Pool<sqlx::Any>, sqlx::Error>{
    AnyPoolOptions::new()
        .max_connections(5)
        .connect(&env::var("SOURCE_DB")
            .expect("Env Variable SOURCE_DB is missing "))
        .await
}