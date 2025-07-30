use blake3::Hash;

use novasmt::{NodeStore, Tree};
use thiserror::Error;

use crate::{Address, ChainId, ContractCode, Quantity, TokenId, Transaction};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    pub chain_id: ChainId,
    pub prev: Hash,
    pub height: u64,
    pub gas_price: Quantity, // the price of 1M gas
    pub state: Hash,
}

impl Header {
    /// Given the gas price in this header, calculate the exact amount of gas available to a transaction with the given transaction fee.
    pub fn fee_to_gas(&self, fee: Quantity) -> u64 {
        // Compute floor(fee * 1_000_000 / gas_price) using integer-only arithmetic without overflow.
        let price = self.gas_price.0;
        let fee0 = fee.0;
        let whole = fee0 / price;
        let rem = fee0 % price;
        // price is per 1,000,000 gas units
        const GAS_DENOMINATOR: u128 = 1_000_000;
        let gas_hi = whole * GAS_DENOMINATOR;
        let gas_lo = rem * GAS_DENOMINATOR / price;
        let gas = gas_hi + gas_lo;
        gas as u64
    }
}

#[derive(Clone)]
pub struct StateHandle<'a, S: NodeStore> {
    last_header: Header,
    state: Tree<'a, S>,
}

impl<'a, S: NodeStore> StateHandle<'a, S> {
    /// Applies the transaction, returning the new state handle.
    pub fn apply_tx(mut self, txn: &Transaction) -> Result<Self, ApplyTxError> {
        let mut gas_left = self.last_header.fee_to_gas(txn.fee);

        if self.last_header.chain_id != txn.chain_id {
            return Err(ApplyTxError::WrongNetId);
        }

        let from_contract: ContractCode = self.load_contract(txn.from)?;
        let to_contract: ContractCode = self.load_contract(txn.to)?;
        if !from_contract
            .execute(
                &self.last_header,
                &mut self.state,
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
                &mut self.state,
                Some(txn),
                0x01,
                &txn.call_data,
                &mut gas_left,
            )
            .ok_or(ApplyTxError::OutOfGas)?
        {
            return Err(ApplyTxError::FromFailed);
        }
        // execute the fees.
        let from_mel_balance = self.get_balance(txn.from, TokenId::MEL)?;
        if txn.fee > from_mel_balance {
            return Err(ApplyTxError::OutOfMoney(TokenId::MEL));
        }
        self.set_balance(txn.from, TokenId::MEL, from_mel_balance - txn.fee)?;

        // execute the asset movement.
        for (&token, &quant) in txn.assets.iter() {
            gas_left = gas_left.checked_sub(200).ok_or(ApplyTxError::OutOfGas)?;
            let from_balance = self.get_balance(txn.from, token)?;
            let to_balance = self.get_balance(txn.to, token)?;
            if quant > from_balance {
                return Err(ApplyTxError::OutOfMoney(token));
            }
            let new_from_balance = from_balance - quant;
            let new_to_balance = to_balance + quant;
            self.set_balance(txn.from, token, new_from_balance)?;
            self.set_balance(txn.to, token, new_to_balance)?;
        }
        Ok(self)
    }

    fn load_contract(&self, addr: Address) -> Result<ContractCode, ApplyTxError> {
        bcs::from_bytes(&self.state.get(addr.into())?)
            .map_err(|e| ApplyTxError::StateCorruption(e.into()))
    }

    fn get_balance(&self, addr: Address, tok: TokenId) -> Result<Quantity, ApplyTxError> {
        let balance = self.state.get(addr.token_state_key(tok).0)?;
        if balance.is_empty() {
            return Err(ApplyTxError::OutOfMoney(tok));
        }
        bcs::from_bytes(&balance).map_err(|e| ApplyTxError::StateCorruption(anyhow::anyhow!(e)))
    }

    fn set_balance(
        &mut self,
        addr: Address,
        tok: TokenId,
        balance: Quantity,
    ) -> Result<(), ApplyTxError> {
        self.state = self.state.clone().with(
            addr.token_state_key(tok).0,
            &bcs::to_bytes(&balance).unwrap(),
        )?;
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum ApplyTxError {
    #[error("out of money for token ID {0:?}")]
    OutOfMoney(TokenId),

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
