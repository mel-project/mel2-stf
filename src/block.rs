use novasmt::{NodeStore, Tree};
use serde::{Deserialize, Serialize};
use tmelcrypt::HashVal;

use crate::{ApplyTxError, Header, StateHandle, Transaction};

#[derive(Serialize, Deserialize, Clone)]
pub struct Block {
    pub header: Header,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn next_block<'a, S: NodeStore>(&self, store: &'a S) -> InProgressBlock<'a, S> {
        let state = Tree::open(store, self.header.state.0);

        InProgressBlock {
            handle: StateHandle {
                last_header: self.header,
                state,
            },
            transactions: vec![],
        }
    }
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

    /// "Seals" this block into a proper block.
    pub fn seal(self) -> Block {
        // TODO: per-block maintenance (melswap, coinbase, etc)
        Block {
            header: Header {
                chain_id: self.handle.last_header.chain_id,
                prev: tmelcrypt::hash_single(bcs::to_bytes(&self.handle.last_header).unwrap()),
                height: self.handle.last_header.height + 1,
                gas_price: self.handle.last_header.gas_price,
                state: HashVal(self.handle.state.root_hash()),
            },
            transactions: self.transactions,
        }
    }
}
