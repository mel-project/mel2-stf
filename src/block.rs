use novasmt::{NodeStore, Tree};
use serde::{Deserialize, Serialize};
use tmelcrypt::HashVal;

use crate::{Address, ApplyTxError, ChainId, Header, Quantity, StateHandle, Transaction};

#[derive(Serialize, Deserialize, Clone)]
/// A block, committing to a certain state in the header, and including the list of transactions since the last block height.
pub struct Block {
    pub header: Header,
    pub transactions: Vec<Transaction>,
    pub seal_info: SealingInfo,
}

impl Block {
    /// The betanet genesis block.
    pub fn betanet_genesis() -> Self {
        Self {
            header: Header {
                chain_id: ChainId::BETANET,
                prev: HashVal::default(),
                height: 0,
                gas_price: Quantity(1_000_000),
                state: HashVal::default(),
            },
            transactions: vec![],
            seal_info: SealingInfo {
                proposer: Address::ZERO,
                new_gas_price: Quantity(1_000_000),
            },
        }
    }

    /// The testnet genesis block.
    pub fn testnet_genesis() -> Self {
        let mut b = Self::betanet_genesis();
        b.header.chain_id = ChainId::TESTNET;
        b
    }

    /// Given this block, and a node store, create an in-progress structure for the next block.
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

/// A block that is "in-progress", not yet sealed into an actual block yet.
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
    pub fn seal(self, seal_info: SealingInfo) -> Block {
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
            seal_info,
        }
    }
}

/// The "sealing info" of the block, which records the discretionary actions of the block producer to seal this block.
#[derive(Serialize, Deserialize, Clone)]
pub struct SealingInfo {
    pub proposer: Address,
    pub new_gas_price: Quantity,
}
