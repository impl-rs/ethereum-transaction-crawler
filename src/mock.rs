use anyhow::Result;
use ethers::prelude::{
    providers::{MockProvider, Provider},
    types::{Address, Block, Transaction, H256, U64},
};
use std::sync::Arc;

pub const ADDRESS: &str = "0xaa7a9ca87d3694b5755f213b5d04094b8d0f0a6f";
pub const ZERO_VALUE: &str = "0.000000000000000000";

pub fn setup_provider(mock_provider: MockProvider) -> Arc<Provider<MockProvider>> {
    Arc::new(Provider::new(mock_provider))
}

pub fn get_mock(address: Address) -> Result<MockProvider> {
    let mock_provider = MockProvider::new();

    // push result for third transaction
    let transaction: Transaction = Transaction::default();
    mock_provider.push(Some(transaction))?;

    // push result for second transaction
    let mut transaction: Transaction = Transaction::default();
    transaction.to = Some(address);
    mock_provider.push(Some(transaction))?;

    // push result for first transaction
    let mut transaction: Transaction = Transaction::default();
    transaction.from = address;
    mock_provider.push(Some(transaction))?;

    // push result for third block
    let mut block: Block<H256> = Block::default();
    block.transactions = vec![H256::zero()];
    mock_provider.push(block)?;

    // push result for second block
    let mut block: Block<H256> = Block::default();
    block.transactions = vec![H256::zero(), H256::zero()];
    mock_provider.push(block)?;

    // push result for first block
    let block: Block<H256> = Block::default();
    mock_provider.push(block)?;

    // push result for get_block_number
    mock_provider.push(U64::from(3))?;

    Ok(mock_provider)
}
