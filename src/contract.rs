use serde::{Deserialize, Serialize};
use tmelcrypt::Ed25519PK;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy, Eq, Hash)]
pub struct Address([u8; 32]);

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum ContractCode {
    Ed25519PK(Ed25519PK),
}
