use novasmt::{NodeStore, Tree};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tmelcrypt::HashVal;

use crate::{Address, ApplyTxError, ChainId, Header, Quantity, StateHandle, TokenId, Transaction};

#[derive(Serialize, Deserialize, Clone, Debug)]
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

    /// Applies and validates the next block.
    pub fn apply_and_validate(
        &self,
        block: &Block,
        store: &impl NodeStore,
    ) -> Result<Block, ApplyBlockError> {
        let mut next = self.next_block(store);
        for txn in block.transactions.iter() {
            next.apply_tx(txn.clone())?;
        }
        let next = next.sealed(block.seal_info)?;
        if next.header != block.header {
            return Err(ApplyBlockError::HeaderMismatch);
        }
        Ok(next)
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
    pub fn sealed(mut self, seal_info: SealingInfo) -> Result<Block, SealBlockError> {
        // TODO: a proper coinbase system rather than just giving 1 MEL to the proposer
        let last_gas_price = self.handle.last_header.gas_price.0;
        if seal_info.new_gas_price.0 > last_gas_price * 10 / 9 + 1
            || seal_info.new_gas_price.0 < last_gas_price * 9 / 10
        {
            return Err(SealBlockError::GasPriceOutOfRange);
        }

        let new_prop_balance = self
            .handle
            .get_balance(seal_info.proposer, TokenId::MEL)
            .unwrap_or_default()
            + Quantity(1_000_000);
        self.handle
            .set_balance(seal_info.proposer, TokenId::MEL, new_prop_balance)?;
        let header = Header {
            chain_id: self.handle.last_header.chain_id,
            prev: tmelcrypt::hash_single(bcs::to_bytes(&self.handle.last_header).unwrap()),
            height: self.handle.last_header.height + 1,
            gas_price: seal_info.new_gas_price,
            state: HashVal(self.handle.state.root_hash()),
        };
        self.handle.state.commit().unwrap();
        Ok(Block {
            header,
            transactions: self.transactions,
            seal_info,
        })
    }
}

#[derive(Error, Debug)]
pub enum SealBlockError {
    #[error("new gas price out of range")]
    GasPriceOutOfRange,
    #[error("applying coinbase failed: {0:?}")]
    CoinbaseFailed(#[from] ApplyTxError),
}

#[derive(Error, Debug)]
pub enum ApplyBlockError {
    #[error("applying transaction failed: {0:?}")]
    ApplyTxFailed(#[from] ApplyTxError),
    #[error("sealing failed: {0:?}")]
    SealFailed(#[from] SealBlockError),
    #[error("header mismatch")]
    HeaderMismatch,
}

/// The "sealing info" of the block, which records the discretionary actions of the block producer to seal this block.
#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
pub struct SealingInfo {
    pub proposer: Address,
    pub new_gas_price: Quantity,
}
