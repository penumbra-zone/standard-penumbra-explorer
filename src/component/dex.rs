use std::collections::HashSet;

use anyhow::anyhow;
use axum::async_trait;
use penumbra_asset::asset::Id as AssetId;
use penumbra_num::Amount;
use penumbra_proto::{event::ProtoEvent, penumbra::core::component::dex::v1 as pb};
use pindexer::{AppView, ContextualizedEvent, PgTransaction};
use sqlx::{PgPool, Postgres, Transaction};

use crate::sql::{SQLAmount, SQLAssetId};

/// One of the possible events that we care about.
#[derive(Clone, Copy, Debug)]
enum Event {
    /// A parsed version of [pb::EventValueCircuitBreakerCredit].
    CircuitBreakerCredit {
        asset_id: AssetId,
        previous_balance: Amount,
        new_balance: Amount,
    },
    /// A parsed version of [pb::EventValueCircuitBreakerDebit]
    CircuitBreakerDebit {
        asset_id: AssetId,
        previous_balance: Amount,
        new_balance: Amount,
    },
}

impl Event {
    const NAMES: [&'static str; 2] = [
        "penumbra.core.component.dex.v1.EventValueCircuitBreakerCredit",
        "penumbra.core.component.dex.v1.EventValueCircuitBreakerDebit",
    ];

    /// Index this event, using the handle to the postgres transaction.
    async fn index<'d>(&self, dbtx: &mut Transaction<'d, Postgres>) -> anyhow::Result<()> {
        match *self {
            Event::CircuitBreakerCredit {
                asset_id,
                previous_balance,
                new_balance,
            } => {
                let amount = new_balance.checked_sub(&previous_balance).ok_or(anyhow!(
                    "balance decreased after dex credit: previous: {}, new: {}",
                    previous_balance,
                    new_balance
                ))?;
                sqlx::query(
                    r#"
                INSERT INTO dex_value_circuit_breaker_change
                VALUES ($1, $2);
                "#,
                )
                .bind(SQLAssetId::from(asset_id))
                .bind(SQLAmount::from(amount))
                .execute(dbtx.as_mut())
                .await?;
                Ok(())
            }
            Event::CircuitBreakerDebit {
                asset_id,
                previous_balance,
                new_balance,
            } => {
                let amount = previous_balance.checked_sub(&new_balance).ok_or(anyhow!(
                    "balance increased after dex credit: previous: {}, new: {}",
                    previous_balance,
                    new_balance
                ))?;
                sqlx::query(
                    r#"
                INSERT INTO dex_value_circuit_breaker_change
                VALUES ($1, -$2);
                "#,
                )
                .bind(SQLAssetId::from(asset_id))
                .bind(SQLAmount::from(amount))
                .execute(dbtx.as_mut())
                .await?;
                Ok(())
            }
        }
    }
}

impl<'a> TryFrom<&'a ContextualizedEvent> for Event {
    type Error = anyhow::Error;

    fn try_from(event: &'a ContextualizedEvent) -> Result<Self, Self::Error> {
        match event.event.kind.as_str() {
            // Credit
            x if x == Event::NAMES[0] => {
                let pe = pb::EventValueCircuitBreakerCredit::from_event(event.as_ref())?;
                let asset_id =
                    AssetId::try_from(pe.asset_id.ok_or(anyhow!("event missing asset_id"))?)?;
                let previous_balance = Amount::try_from(
                    pe.previous_balance
                        .ok_or(anyhow!("event missing previous_balance"))?,
                )?;
                let new_balance =
                    Amount::try_from(pe.new_balance.ok_or(anyhow!("event missing new_balance"))?)?;
                Ok(Self::CircuitBreakerCredit {
                    asset_id,
                    previous_balance,
                    new_balance,
                })
            }
            // Debit
            x if x == Event::NAMES[1] => {
                let pe = pb::EventValueCircuitBreakerDebit::from_event(event.as_ref())?;
                let asset_id =
                    AssetId::try_from(pe.asset_id.ok_or(anyhow!("event missing asset_id"))?)?;
                let previous_balance = Amount::try_from(
                    pe.previous_balance
                        .ok_or(anyhow!("event missing previous_balance"))?,
                )?;
                let new_balance =
                    Amount::try_from(pe.new_balance.ok_or(anyhow!("event missing new_balance"))?)?;
                Ok(Self::CircuitBreakerDebit {
                    asset_id,
                    previous_balance,
                    new_balance,
                })
            }
            x => Err(anyhow!(format!("unrecognized event kind: {x}"))),
        }
    }
}

#[derive(Debug)]
pub struct Component {
    event_strings: HashSet<&'static str>,
}

impl Component {
    pub fn new() -> Self {
        let event_strings = Event::NAMES.into_iter().collect();
        Self { event_strings }
    }

    pub fn attach_to_indexer(self, indexer: pindexer::Indexer) -> pindexer::Indexer {
        indexer.with_index(self)
    }
}

#[async_trait]
impl AppView for Component {
    async fn init_chain(
        &self,
        dbtx: &mut PgTransaction,
        _app_state: &serde_json::Value,
    ) -> anyhow::Result<()> {
        sqlx::query(include_str!("dex.sql"))
            .execute(dbtx.as_mut())
            .await?;
        Ok(())
    }

    fn is_relevant(&self, type_str: &str) -> bool {
        self.event_strings.contains(type_str)
    }

    #[tracing::instrument(skip_all, fields(height = event.block_height))]
    async fn index_event(
        &self,
        dbtx: &mut PgTransaction,
        event: &ContextualizedEvent,
        _src_db: &PgPool,
    ) -> anyhow::Result<()> {
        Event::try_from(event)?.index(dbtx).await
    }
}
