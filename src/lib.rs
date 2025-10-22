mod block;
mod contract;
mod state;
mod transaction;

pub use block::*;
pub use contract::*;
pub use state::*;
pub use transaction::*;

#[cfg(test)]
mod tests {
    use novasmt::InMemoryStore;

    use crate::{Address, Block};

    #[test]
    pub fn testnet() {
        let testnet = Block::testnet_genesis();
        let store = InMemoryStore::default();
        let mut current = testnet.clone();
        for _ in 0..1000 {
            current = current
                .next_block(&store)
                .sealed(crate::SealingInfo {
                    proposer: Address::ZERO,
                    new_gas_price: crate::Quantity(1_000_000),
                })
                .unwrap();
        }
        dbg!(current.header);
    }
}
