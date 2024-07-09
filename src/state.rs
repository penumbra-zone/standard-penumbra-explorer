use sqlx::PgPool;

/// Represents the state of our application.
#[derive(Debug, Clone)]
pub struct AppState {
    /// The connection pool for postgres.
    pool: PgPool,
}

impl AppState {
    /// Create an instance of this state.
    ///
    /// - `db_url` is used to connect to our postgres database.
    pub async fn create(db_url: &str) -> anyhow::Result<Self> {
        let pool = PgPool::connect(db_url).await?;
        Ok(Self { pool })
    }

    /// Get the database pool associated with this state.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}
