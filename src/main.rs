pub mod crawler;
#[cfg(test)]
pub mod mock;
pub mod templates;
use crate::crawler::{Crawler, Wallet};
use crate::templates::{body, form};
use axum::{
    extract::{Query, State},
    routing::get,
    Router,
};
use ethers::prelude::providers::Http;
use ethers::prelude::providers::{JsonRpcClient, Provider};
use maud::{html, Markup};
use std::env::var;
use std::net::SocketAddr;
use std::sync::Arc;
use tower::limit::ConcurrencyLimitLayer;

fn get_provider() -> Provider<impl JsonRpcClient> {
    let http_provider = var("HTTP_PROVIDER").expect("HTTP_PROVIDER not set");
    Provider::<Http>::try_from(http_provider).expect("could not instantiate HTTP Provider")
}

#[tokio::main]
async fn main() {
    // initialize provider
    let provider = Arc::new(get_provider());
    // restrict to one concurrent request at the time
    let middleware = tower::ServiceBuilder::new().layer(ConcurrencyLimitLayer::new(1));

    let app = Router::new()
        .route("/", get(handler))
        .layer(middleware)
        .with_state(provider);

    let socket_address = SocketAddr::from(([127, 0, 0, 1], 8000));

    axum::Server::bind(&socket_address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler(
    State(provider): State<Arc<Provider<impl JsonRpcClient>>>,
    maybe_wallet: Option<Query<Wallet>>,
) -> Markup {
    let title = "Ethereum transaction crawler";
    if let Some(wallet) = maybe_wallet {
        let crawler = Crawler::new(provider, wallet.clone());
        let maybe_transactions = crawler.get_transactions().await;
        body(
            title,
            html! {
                (form())
                div {
                    "Transactions: "
                    table {
                        tr {
                            th { "From" }
                            th { "To" }
                            th { "Value" }
                        }
                        @if let Ok(transactions) = maybe_transactions {
                            @for transaction in &transactions {
                                tr {
                                    td { (transaction.from) }
                                    @if let Some(transaction_to) = transaction.to {
                                        td { (transaction_to) }
                                    } @else {
                                        td { "-" }
                                    }
                                    td { (transaction.value) }
                                }
                            }
                        }
                    }
                }
            },
        )
    } else {
        body(
            title,
            html! {
                (form())
                div {
                    "Select a wallet and block number to search"
                }
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{get_mock, setup_provider, ADDRESS};
    use anyhow::Result;
    use ethers::prelude::types::Address;
    use maud::PreEscaped;

    #[tokio::test]
    async fn test_handler() -> Result<()> {
        let address = ADDRESS.parse::<Address>()?;

        let wallet = Wallet {
            address: ADDRESS.to_string(),
            block: 1,
        };
        let PreEscaped(markup) = handler(
            State(setup_provider(get_mock(address)?)),
            Some(Query(wallet)),
        )
        .await;

        assert!(
            markup.contains("<tr><td>0xaa7a…0a6f</td><td>-</td><td>0.000000000000000000</td></tr>")
        );
        assert!(markup.contains(
            "<tr><td>0x0000…0000</td><td>0xaa7a…0a6f</td><td>0.000000000000000000</td></tr>"
        ));

        Ok(())
    }
}
