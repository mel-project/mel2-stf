use novasmt::NodeStore;
use serde::{Deserialize, Serialize};

use crate::{ApplyTxError, Header, StateHandle, Transaction};

#[derive(Serialize, Deserialize, Clone)]
pub struct Block {
    pub header: Header,
    pub transactions: Vec<Transaction>,
}

pub struct InProgressBlock<'a, S: NodeStore> {
    handle: StateHandle<'a, S>,
    transactions: Vec<Transaction>,
}

impl<'a, S: NodeStore> InProgressBlock<'a, S> {
    /// Applies the transaction to this in-progress block. This method is atomic; if the transaction failed to apply, we revert to a safe state.
    pub fn apply_tx(&mut self, txn: Transaction) -> Result<(), ApplyTxError> {
        let new_handle = self.handle.clone().apply_tx(&txn)?;
        self.handle = new_handle;
        self.transactions.push(txn.clone());
        Ok(())
    }
}
