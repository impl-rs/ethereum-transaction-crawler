use anyhow::Result;
use axum::extract::Query;

use ethers::{
    abi::Address,
    prelude::{
        providers::{JsonRpcClient, Middleware, Provider},
        types::H256,
    },
    types::Transaction,
    utils::format_ether,
};
use futures::{stream, StreamExt};
use serde::Deserialize;
use std::sync::Arc;
use std::time::Instant;

const BUFFER_SIZE: usize = 100;

#[derive(Deserialize, Debug, Clone)]
pub struct Wallet {
    pub address: String,
    pub block: u64,
}

pub struct Crawler<T: JsonRpcClient> {
    provider: Arc<Provider<T>>,
    wallet: Wallet,
}

async fn fetch_block(provider: Arc<Provider<impl JsonRpcClient>>, block_number: u64) -> Vec<H256> {
    println!("Fetching block {}", block_number);
    let maybe_block = provider.get_block(block_number).await;

    match maybe_block {
        Ok(Some(block)) => block.transactions,
        _ => vec![],
    }
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct AddressTransaction {
    pub from: Address,
    pub to: Option<Address>,
    pub value: String,
}

fn is_address_transaction(
    address: Address,
    transaction: Transaction,
) -> Option<AddressTransaction> {
    if transaction.from == address || transaction.to == Some(address) {
        Some(AddressTransaction {
            from: transaction.from,
            to: transaction.to,
            value: format_ether(transaction.value),
        })
    } else {
        None
    }
}

async fn fetch_transaction(
    provider: Arc<Provider<impl JsonRpcClient>>,
    address: Address,
    tx: H256,
) -> Option<AddressTransaction> {
    println!("Fetching transaction {}", tx);
    let maybe_transaction = provider.get_transaction(tx).await;

    match maybe_transaction {
        Ok(Some(transaction)) => is_address_transaction(address, transaction),
        _ => None,
    }
}

impl<T: JsonRpcClient> Crawler<T> {
    pub fn new(provider: Arc<Provider<T>>, Query(wallet): Query<Wallet>) -> Self {
        Self { provider, wallet }
    }
    pub async fn get_transactions(self) -> Result<Vec<AddressTransaction>> {
        let Crawler {
            provider,
            wallet:
                Wallet {
                    address,
                    block: from_block,
                },
            ..
        } = self;
        let address = address.parse::<Address>()?;

        let to_block = provider.get_block_number().await?.as_u64();

        // For benchmarking
        let now = Instant::now();

        let address_transactions: Vec<AddressTransaction> = stream::iter(from_block..=to_block)
            .map(|block_number| fetch_block(provider.clone(), block_number))
            .buffered(BUFFER_SIZE)
            .flat_map(stream::iter)
            .map(|tx| fetch_transaction(provider.clone(), address, tx))
            .buffered(BUFFER_SIZE)
            .filter_map(|tx| async { tx })
            .collect()
            .await;

        let elapsed = now.elapsed();
        println!("Elapsed: {:.2?}", elapsed);

        Ok(address_transactions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{get_mock, setup_provider, ADDRESS, ZERO_VALUE};
    use ethers::prelude::{
        providers::MockProvider,
        types::{Block, Transaction, H256},
    };

    #[tokio::test]
    async fn test_fetch_block() -> Result<()> {
        let mock_provider = MockProvider::new();

        let mut block: Block<H256> = Block::default();

        block.transactions = vec![H256::zero()];
        mock_provider.push(block)?;

        let transactions = fetch_block(setup_provider(mock_provider), 1).await;

        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0], H256::zero());

        Ok(())
    }

    async fn get_test_transaction(
        address: Address,
        tx: H256,
        update_transaction: impl Fn(&mut Transaction) -> (),
    ) -> Result<Option<AddressTransaction>> {
        let mock_provider = MockProvider::new();

        let mut transaction: Transaction = Transaction::default();

        update_transaction(&mut transaction);

        mock_provider.push(transaction)?;

        let transaction = fetch_transaction(setup_provider(mock_provider), address, tx).await;

        Ok(transaction)
    }

    #[tokio::test]
    async fn test_fetch_transaction() -> Result<()> {
        let address = ADDRESS.parse::<Address>()?;
        let tx = H256::zero();

        // Should return None when address is not in from or to of transaction
        let transaction = get_test_transaction(address, tx, |_| {}).await?;
        assert_eq!(transaction, None);

        // Should return Some when address is in from
        let transaction = get_test_transaction(address, tx, |transaction| {
            transaction.from = address;
        })
        .await?;
        assert_eq!(
            transaction,
            Some(AddressTransaction {
                from: address,
                to: None,
                value: ZERO_VALUE.into()
            })
        );

        // Should return Some when address is in to
        let transaction = get_test_transaction(address, tx, |transaction| {
            transaction.to = Some(address);
        })
        .await?;
        assert_eq!(
            transaction,
            Some(AddressTransaction {
                from: Address::zero(),
                to: Some(address),
                value: ZERO_VALUE.into()
            })
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_crawler() -> Result<()> {
        let address = ADDRESS.parse::<Address>()?;

        let mock_provider = get_mock(address)?;

        let crawler = Crawler::new(
            setup_provider(mock_provider),
            Query(Wallet {
                address: ADDRESS.into(),
                block: 1,
            }),
        );

        let transactions = crawler.get_transactions().await?;
        assert_eq!(
            transactions,
            [
                AddressTransaction {
                    from: address,
                    to: None,
                    value: ZERO_VALUE.into()
                },
                AddressTransaction {
                    from: Address::zero(),
                    to: Some(address,),
                    value: ZERO_VALUE.into()
                },
            ]
        );

        Ok(())
    }
}
