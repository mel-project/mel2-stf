use derive_more::{From, Into};
use novasmt::{NodeStore, Tree};
use serde::{Deserialize, Serialize};

use tmelcrypt::Ed25519PK;

use crate::{Header, Transaction};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy, Eq, Hash, From, Into)]
pub struct Address([u8; 32]);

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Eq, Hash)]
pub enum ContractCode {
    Ed25519PK(Ed25519PK),
}

impl ContractCode {
    /// Executes a contract in its proper context.
    pub fn execute<'a, S: NodeStore>(
        &self,
        last_header: &Header,
        state: &Tree<'a, S>,
        calling_tx: Option<&Transaction>,
        entry: u64,
        data: &[u8],
        gas: &mut u64,
    ) -> Option<bool> {
        todo!()
    }
}
