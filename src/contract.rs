use derive_more::{From, Into};
use novasmt::{NodeStore, Tree};
use serde::{Deserialize, Serialize};

use tmelcrypt::{Ed25519PK, HashVal};

use crate::{Header, TokenId, Transaction};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy, Eq, Hash, From, Into)]
pub struct Address([u8; 32]);

impl Address {
    /// The state key where a token balance is held.
    pub fn token_state_key(&self, token: TokenId) -> HashVal {
        tmelcrypt::hash_single(&bcs::to_bytes(&(self.0, b"token", token)).unwrap())
    }

    /// The state key where a blob is held.
    pub fn blob_state_key(&self, token: TokenId) -> HashVal {
        tmelcrypt::hash_single(&bcs::to_bytes(&(self.0, b"blob", token)).unwrap())
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Eq, Hash)]
pub enum ContractCode {
    Ed25519PK(Ed25519PK),
}

impl ContractCode {
    /// Executes a contract in its proper context.
    pub fn execute<'a, S: NodeStore>(
        &self,
        last_header: &Header,
        state: &mut Tree<'a, S>,
        calling_tx: Option<&Transaction>,
        entry: u64,
        data: &[u8],
        gas: &mut u64,
    ) -> Option<bool> {
        todo!()
    }
}
