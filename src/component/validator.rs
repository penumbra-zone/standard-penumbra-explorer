use decaf377_rdsa::VerificationKeyBytes;
use penumbra_keys::address::Address;
use penumbra_stake::{validator, IdentityKey};
use serde::{self, Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use sqlx::{postgres::PgRow, PgPool};

/// Represents a very basic view of a Validator
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorSummary {
    /// The name of the validator
    pub name: String,
    /// The identity key of the validator
    #[serde_as(as = "DisplayFromStr")]
    pub identity: IdentityKey,
    /// The voting power of the validator
    pub voting_power: i64,
}

impl<'r> sqlx::FromRow<'r, PgRow> for ValidatorSummary {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        let x: (&'r str, [u8; 32], i64) = sqlx::FromRow::from_row(row)?;
        Ok(ValidatorSummary {
            name: x.0.to_string(),
            identity: IdentityKey(VerificationKeyBytes::from(x.1)),
            voting_power: x.2,
        })
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingStream {
    #[serde_as(as = "DisplayFromStr")]
    pub address: Address,
    pub rate_bps: u16,
}

/// A full description of a validator.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    /// The identity key of the validator
    #[serde_as(as = "DisplayFromStr")]
    pub identity: IdentityKey,
    /// The name of the validator
    pub name: String,
    /// The voting power of the validator
    pub voting_power: i64,
    /// A short of what the validator is
    pub description: String,
    /// A link to the validator's website
    pub website: String,
    /// Whether or not the validator is enabled
    pub enabled: bool,
    /// Where the commission, if any, of the validator goes
    pub funding_streams: Vec<FundingStream>,
    /// The contribution to the community pool
    pub community_pool_rate_bps: u16,
}

impl<'r> sqlx::FromRow<'r, PgRow> for Validator {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        let x: ([u8; 32], &'r str, i64, &'r str) = sqlx::FromRow::from_row(row)?;

        let identity = IdentityKey(VerificationKeyBytes::from(x.0));
        let name = x.1.to_string();
        let voting_power = x.2;
        let definition: validator::Validator =
            serde_json::from_str(x.3).map_err(|e| sqlx::Error::ColumnDecode {
                index: "3".to_string(),
                source: Box::new(e),
            })?;
        let description = definition.description;
        let website = definition.website;
        let enabled = definition.enabled;

        let (community_pool_rate_bps, funding_streams) = {
            let mut community_pool_rate_bps = 0u16;
            let mut funding_streams = Vec::new();
            for x in definition.funding_streams.iter() {
                match x {
                    penumbra_stake::FundingStream::ToAddress { address, rate_bps } => {
                        funding_streams.push(FundingStream {
                            address: address.clone(),
                            rate_bps: *rate_bps,
                        })
                    }
                    penumbra_stake::FundingStream::ToCommunityPool { rate_bps } => {
                        community_pool_rate_bps += rate_bps
                    }
                }
            }
            funding_streams.sort_by_key(|x| std::cmp::Reverse(x.rate_bps));
            (community_pool_rate_bps, funding_streams)
        };

        Ok(Self {
            identity,
            name,
            voting_power,
            description,
            website,
            enabled,
            funding_streams,
            community_pool_rate_bps,
        })
    }
}

/// A component for indexing and retrieving information about blocks.
#[derive(Debug, Clone, Copy)]
pub struct Component {}

impl Component {
    pub const TEMPLATES: [(&'static str, &'static str); 2] = [
        (
            "validators",
            include_str!("../../templates/validators.html"),
        ),
        ("validator", include_str!("../../templates/validator.html")),
    ];

    pub fn new() -> Self {
        Self {}
    }

    pub fn attach_to_indexer(self, indexer: pindexer::Indexer) -> pindexer::Indexer {
        indexer.with_index(pindexer::stake::ValidatorSet {})
    }

    /// Fetch a list of validators, in descending voting power.
    pub async fn validators(pool: &PgPool) -> anyhow::Result<Vec<ValidatorSummary>> {
        Ok(sqlx::query_as(
            "SELECT name, ik, voting_power FROM stake_validator_set ORDER BY voting_power DESC;",
        )
        .fetch_all(pool)
        .await?)
    }

    /// Fetch a specific validator, by identity key
    pub async fn validator(pool: &PgPool, identity: &IdentityKey) -> anyhow::Result<Validator> {
        Ok(sqlx::query_as(
            "SELECT ik, name, voting_power, definition FROM stake_validator_set WHERE ik = $1;",
        )
        .bind(identity.to_bytes())
        .fetch_one(pool)
        .await?)
    }
}
