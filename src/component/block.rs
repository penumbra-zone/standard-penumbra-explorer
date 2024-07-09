use axum::async_trait;
use pindexer::{AppView, ContextualizedEvent, PgPool, PgTransaction};
use sqlx::types::chrono::{DateTime, Utc};

/// A component for indexing and retrieving information about blocks.
#[derive(Debug, Clone, Copy)]
pub struct Block {}

impl Block {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl AppView for Block {
    async fn init_chain(&self, dbtx: &mut PgTransaction) -> Result<(), anyhow::Error> {
        sqlx::query(include_str!("block.sql"))
            .execute(dbtx.as_mut())
            .await?;
        Ok(())
    }

    fn is_relevant(&self, type_str: &str) -> bool {
        type_str == "block"
    }

    async fn index_event(
        &self,
        dbtx: &mut PgTransaction,
        event: &ContextualizedEvent,
        src_db: &PgPool,
    ) -> Result<(), anyhow::Error> {
        let row: (i64, DateTime<Utc>, i64) = sqlx::query_as(
            r#"
SELECT
    blocks.height,
    blocks.created_at,
    (SELECT count(*) FROM tx_results WHERE tx_results.block_id = blocks.rowid)
FROM 
    events JOIN blocks on events.block_id = blocks.rowid
WHERE 
    events.rowid = $1;"#,
        )
        .bind(event.local_rowid)
        .fetch_one(src_db)
        .await?;

        sqlx::query(
            r#"
INSERT INTO block
VALUES ($1, $3, $2);"#,
        )
        .bind(row.0)
        .bind(row.1)
        .bind(row.2)
        .execute(dbtx.as_mut())
        .await?;

        Ok(())
    }
}
