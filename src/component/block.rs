use axum::async_trait;
use pindexer::{AppView, ContextualizedEvent, PgPool, PgTransaction};
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};

use crate::pagination::Pagination;

/// Implement a block
#[derive(Debug, Clone, Copy, sqlx::FromRow, Serialize, Deserialize)]
pub struct Block {
    pub height: i64,
    pub transaction_count: i64,
    pub created_at: DateTime<Utc>,
}

/// A component for indexing and retrieving information about blocks.
#[derive(Debug, Clone, Copy)]
pub struct Component {}

impl Component {
    pub const TEMPLATE: (&'static str, &'static str) =
        ("block", include_str!("../../templates/blocks.html"));

    pub fn new() -> Self {
        Self {}
    }

    /// Fetch a list of blocks.
    ///
    /// This will be sorted in reverse reverse order, by default.
    pub async fn blocks(pool: &PgPool, pagination: &Pagination<i64>) -> anyhow::Result<Vec<Block>> {
        Ok(sqlx::query_as(
            "SELECT * FROM block WHERE height BETWEEN $1 AND $2 ORDER BY height DESC;",
        )
        .bind(pagination.start)
        .bind(pagination.stop)
        .fetch_all(pool)
        .await?)
    }
}

#[async_trait]
impl AppView for Component {
    async fn init_chain(
        &self,
        dbtx: &mut PgTransaction,
        _app_state: &serde_json::Value,
    ) -> Result<(), anyhow::Error> {
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
