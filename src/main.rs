pub mod crawler;
pub mod templates;
use crate::crawler::{Crawler, Wallet};
use crate::templates::{body, form};
use axum::{
    extract::{Query, State},
    routing::get,
    Router,
};
use ethers::prelude::providers::{Http, Provider};
use tower::limit::ConcurrencyLimitLayer;

use maud::{html, Markup};
use std::env::var;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // initialize provider
    let http_provider = var("HTTP_PROVIDER").expect("HTTP_PROVIDER not set");
    let provider =
        Provider::<Http>::try_from(http_provider).expect("could not instantiate HTTP Provider");

    // Restrivct to one concurrent request at the time
    let middleware = tower::ServiceBuilder::new().layer(ConcurrencyLimitLayer::new(1));

    let app = Router::new()
        .route("/", get(root))
        .layer(middleware)
        .with_state(provider);

    let socket_address = SocketAddr::from(([127, 0, 0, 1], 8000));

    axum::Server::bind(&socket_address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root(
    State(provider): State<Provider<Http>>,
    maybe_wallet: Option<Query<Wallet>>,
) -> Markup {
    let title = "Ethereum transaction crawler";
    if let Some(wallet) = maybe_wallet {
        let crawler = Crawler::new(provider, wallet.clone());
        let maybe_transactions = crawler.get_transactions(None).await;
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
