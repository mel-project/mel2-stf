use std::{collections::BTreeMap, fmt::Display};

use bytes::Bytes;
use derive_more::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign, Sum};
use serde::{Deserialize, Serialize};

use crate::contract::Address;

/// Network ID.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NetId(pub u16);

impl NetId {
    pub const BETANET: Self = Self(0x0814);
    pub const TESTNET: Self = Self(0xffff);
}

/// A transaction in the block. Moves assets from one address to another.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Transaction {
    pub netid: NetId,
    pub nonce: u64,
    pub from: Address,
    pub to: Address,
    pub fee: u128,
    pub assets: BTreeMap<TokenId, Quantity>,
    pub data: Bytes,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TokenId(pub u64);

impl TokenId {
    pub const MEL: Self = Self(0);
}

/// Newtype representing a monetary value in microunits. The Display and FromStr implementations divide by 1,000,000 automatically.
#[derive(
    Clone,
    Copy,
    Default,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    Add,
    AddAssign,
    Sub,
    SubAssign,
    Div,
    DivAssign,
    Mul,
    MulAssign,
    Sum,
)]
#[serde(transparent)]
pub struct Quantity(pub u128);

impl Display for Quantity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{:06}",
            self.0 / MICRO_CONVERTER,
            self.0 % MICRO_CONVERTER
        )
    }
}

pub const MICRO_CONVERTER: u128 = 1_000_000;
