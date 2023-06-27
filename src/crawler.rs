use anyhow::Result;
use axum::extract::Query;

use ethers::{
    abi::Address,
    prelude::{
        providers::{Http, Middleware, Provider},
        types::H256,
    },
    types::Transaction,
    utils::format_ether,
};
use futures::{stream, StreamExt};
use serde::Deserialize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;

const BUFFER_SIZE: usize = 100;

#[derive(Deserialize, Debug, Clone)]
pub struct Wallet {
    pub address: String,
    pub block: u64,
}

pub struct Crawler {
    provider: Arc<Provider<Http>>,
    wallet: Wallet,
}

async fn fetch_block(provider: Arc<Provider<Http>>, block_number: u64) -> Vec<H256> {
    let maybe_block = provider.get_block(block_number).await;
    sleep(Duration::from_millis(1000)).await;

    match maybe_block {
        Ok(Some(block)) => block.transactions,
        _ => vec![],
    }
}

#[derive(Deserialize)]
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
    provider: Arc<Provider<Http>>,
    address: Address,
    tx: H256,
) -> Option<AddressTransaction> {
    let maybe_transaction = provider.get_transaction(tx).await;
    sleep(Duration::from_millis(1000)).await;

    match maybe_transaction {
        Ok(Some(transaction)) => is_address_transaction(address, transaction),
        _ => None,
    }
}

impl Crawler {
    pub fn new(provider: Provider<Http>, Query(wallet): Query<Wallet>) -> Self {
        Self {
            provider: Arc::new(provider),
            wallet,
        }
    }
    pub async fn get_transactions(
        self,
        maybe_to_block: Option<u64>,
    ) -> Result<Vec<AddressTransaction>> {
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

        // get latest block if no to_block is provided
        let to_block = if let Some(to_block) = maybe_to_block {
            to_block
        } else {
            provider.get_block_number().await?.as_u64()
        };

        // For benchmarking
        let now = Instant::now();

        let address_transactions: Vec<AddressTransaction> =
            stream::iter(from_block..to_block as u64)
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
