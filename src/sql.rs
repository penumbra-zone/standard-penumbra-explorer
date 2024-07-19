//! This module provides various shims for encoding penumbra domain types into Postgres.
use anyhow::anyhow;
use penumbra_asset::asset::Id as AssetId;
use penumbra_num::Amount;
// This type implements the actual arbitrary number in postgres but as decimal points.
use sqlx::{types::BigDecimal, Decode, Encode, Postgres, Type};
// This type is convertible to that, but doesn't have decimal points.
use num_bigint::{BigInt, Sign};

/// Represents an [Amount] that can fit into a Postgres database.
///
/// This has [From] and [Into] implementations for [Amount] for easy conversion.
#[derive(Debug, Clone, Copy)]
pub struct SQLAmount(Amount);

impl SQLAmount {
    /// Create a new [SQLAmount] from an [Amount].
    pub fn new(amount: Amount) -> Self {
        SQLAmount(amount)
    }

    /// Convert this value into an [Amount]
    pub fn amount(self) -> Amount {
        self.0
    }

    fn to_bigint(self) -> BigInt {
        BigInt::from_bytes_le(Sign::Plus, &self.amount().to_le_bytes())
    }

    fn from_bigint(value: BigInt) -> Option<Self> {
        // Get the bytes only from a positive BigInt
        let bytes = match value.to_bytes_le() {
            (Sign::Plus | Sign::NoSign, bytes) => bytes,
            (Sign::Minus, bytes) => bytes,
        };
        let bytes: [u8; 16] = bytes.try_into().ok()?;
        Some(Self::new(Amount::from_le_bytes(bytes)))
    }
}

impl From<Amount> for SQLAmount {
    fn from(value: Amount) -> Self {
        Self::new(value)
    }
}

impl From<SQLAmount> for Amount {
    fn from(value: SQLAmount) -> Self {
        value.amount()
    }
}

impl<'q> Encode<'q, Postgres> for SQLAmount {
    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull {
        BigDecimal::from(self.to_bigint()).encode_by_ref(buf)
    }
}

impl<'q> Decode<'q, Postgres> for SQLAmount {
    fn decode(
        value: <Postgres as sqlx::database::HasValueRef<'q>>::ValueRef,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let big_decimal = BigDecimal::decode(value)?;
        if !big_decimal.is_integer() {
            return Err(anyhow!("database value is not an integer").into());
        }
        let big_int = big_decimal.as_bigint_and_exponent().0;
        Ok(SQLAmount::from_bigint(big_int)
            .ok_or(anyhow!("failed to convert BigInt into SQLAmount"))?)
    }
}

impl Type<Postgres> for SQLAmount {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        BigDecimal::type_info()
    }
}

/// Represents an [AssetId] that can be serialized and deserialized from SQL easily.
#[derive(Debug, Clone, Copy)]
pub struct SQLAssetId(AssetId);

impl SQLAssetId {
    pub fn new(asset_id: AssetId) -> Self {
        Self(asset_id)
    }

    pub fn asset_id(self) -> AssetId {
        self.0
    }
}

impl From<AssetId> for SQLAssetId {
    fn from(value: AssetId) -> Self {
        Self::new(value)
    }
}

impl From<SQLAssetId> for AssetId {
    fn from(value: SQLAssetId) -> Self {
        value.asset_id()
    }
}

impl<'q> Encode<'q, Postgres> for SQLAssetId {
    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull {
        self.asset_id().to_bytes().encode_by_ref(buf)
    }
}

impl<'q> Decode<'q, Postgres> for SQLAssetId {
    fn decode(
        value: <Postgres as sqlx::database::HasValueRef<'q>>::ValueRef,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let bytes = <[u8; 32]>::decode(value)?;
        let asset_id = AssetId::try_from(bytes.as_slice())?;
        Ok(asset_id.into())
    }
}

impl Type<Postgres> for SQLAssetId {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        <[u8; 32]>::type_info()
    }
}
