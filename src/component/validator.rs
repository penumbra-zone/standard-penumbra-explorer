use decaf377_rdsa::VerificationKeyBytes;
use penumbra_stake::IdentityKey;
use serde::{self, Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use sqlx::{postgres::PgRow, PgPool};

/// Represents a very basic view of a Validator
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    /// The name of the validator
    pub name: String,
    /// The identity key of the validator
    #[serde_as(as = "DisplayFromStr")]
    pub identity: IdentityKey,
    /// The voting power of the validator
    pub voting_power: i64,
}

impl<'r> sqlx::FromRow<'r, PgRow> for Validator {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        let x: (&'r str, [u8; 32], i64) = sqlx::FromRow::from_row(row)?;
        Ok(Validator {
            name: x.0.to_string(),
            identity: IdentityKey(VerificationKeyBytes::from(x.1)),
            voting_power: x.2,
        })
    }
}

/// A component for indexing and retrieving information about blocks.
#[derive(Debug, Clone, Copy)]
pub struct Component {}

impl Component {
    pub const TEMPLATE: (&'static str, &'static str) = (
        "validators",
        include_str!("../../templates/validators.html"),
    );

    pub fn new() -> Self {
        Self {}
    }

    pub fn attach_to_indexer(self, indexer: pindexer::Indexer) -> pindexer::Indexer {
        indexer.with_index(pindexer::stake::ValidatorSet {})
    }

    /// Fetch a list of validators, in descending voting power.
    pub async fn validators(pool: &PgPool) -> anyhow::Result<Vec<Validator>> {
        Ok(
            sqlx::query_as("SELECT name, ik, voting_power FROM stake_validator_set ORDER BY voting_power DESC;")
                .fetch_all(pool)
                .await?,
        )
    }
}
