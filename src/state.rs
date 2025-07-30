use blake3::Hash;

use novasmt::{NodeStore, Tree};
use thiserror::Error;

use crate::{Address, ChainId, ContractCode, Transaction};

pub struct Header {
    pub chain_id: ChainId,
    pub prev: Hash,
    pub height: u64,
    pub state: Hash,
}

pub struct StateHandle<'a, S: NodeStore> {
    last_header: Header,
    state: Tree<'a, S>,
}

impl<'a, S: NodeStore> StateHandle<'a, S> {
    pub fn apply_tx(self, txn: &Transaction) -> Result<Self, ApplyTxError> {
        // TODO gas
        let mut gas_left = 10000;

        if self.last_header.chain_id != txn.chain_id {
            return Err(ApplyTxError::WrongNetId);
        }

        let from_contract: ContractCode = self.load_contract(txn.from)?;
        let to_contract: ContractCode = self.load_contract(txn.to)?;
        if !from_contract
            .execute(
                &self.last_header,
                &self.state,
                Some(txn),
                0x00,
                &txn.auth_data,
                &mut gas_left,
            )
            .ok_or(ApplyTxError::OutOfGas)?
        {
            return Err(ApplyTxError::FromFailed);
        }
        if !to_contract
            .execute(
                &self.last_header,
                &self.state,
                Some(txn),
                0x01,
                &txn.call_data,
                &mut gas_left,
            )
            .ok_or(ApplyTxError::OutOfGas)?
        {
            return Err(ApplyTxError::FromFailed);
        }
        todo!()
    }

    fn load_contract(&self, addr: Address) -> Result<ContractCode, ApplyTxError> {
        bcs::from_bytes(&self.state.get(addr.into())?)
            .map_err(|e| ApplyTxError::StateCorruption(e.into()))
    }
}

#[derive(Error, Debug)]
pub enum ApplyTxError {
    #[error("out of gas")]
    OutOfGas,

    #[error("wrong network ID")]
    WrongNetId,

    #[error("from contract failed to run")]
    FromFailed,

    #[error("to contract failed to run")]
    ToFailed,

    #[error("SMT corruption {0:?}")]
    SmtCorruption(#[from] novasmt::SmtError),

    #[error("State item corruption {0:?}")]
    StateCorruption(#[from] anyhow::Error),
}
