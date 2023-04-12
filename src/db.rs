use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::env;

pub async fn connect() -> sqlx::Result<Pool<Postgres>> {
    let pg_host = env::var("POSTGRES_HOST").expect("POSTGRES_HOST is not set");
    let pg_password = env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD is not set");
    let pg_user = env::var("POSTGRES_USER").unwrap_or("postgres".to_string());
    let pg_db = env::var("POSTGRES_DB").unwrap_or(pg_user.clone());
    Ok(PgPoolOptions::new()
        .max_connections(5)
        .connect(format!("postgres://{pg_user}:{pg_password}@{pg_host}/{pg_db}").as_str())
        .await?)
}
