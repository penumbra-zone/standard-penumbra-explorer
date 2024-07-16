use minijinja::Environment;
use serde::Serialize;
use sqlx::PgPool;

use crate::component;

fn create_environment() -> anyhow::Result<Environment<'static>> {
    let mut environment = Environment::new();

    for (name, file) in [
        component::block::Component::TEMPLATE,
        component::validator::Component::TEMPLATE,
    ]
    .into_iter()
    {
        environment.add_template(name, file)?;
    }

    Ok(environment)
}

/// Represents the state of our application.
#[derive(Debug, Clone)]
pub struct AppState {
    /// The connection pool for postgres.
    pool: PgPool,
    environment: Environment<'static>,
}

impl AppState {
    /// Create an instance of this state.
    ///
    /// - `db_url` is used to connect to our postgres database.
    pub async fn create(db_url: &str) -> anyhow::Result<Self> {
        let pool = PgPool::connect(db_url).await?;
        let environment = create_environment()?;
        Ok(Self { pool, environment })
    }

    /// Get the database pool associated with this state.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Render a template by name
    pub fn render_template<S: Serialize>(&self, name: &str, ctx: S) -> anyhow::Result<String> {
        Ok(self.environment.get_template(name)?.render(ctx)?)
    }
}
