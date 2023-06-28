# Ethereum Transaction Crawler

This is a simple application that allows a user to view transaction data from the Ethereum blockchain. It works by inputting a wallet address and a block number. The crawler will then go through all blocks from the inputted block number to the current block number and find all transactions that involve the inputted wallet address. The application will then display the information about the transactions in a table with to, and from address and the amount of ether transferred.

To get transaction data from a given address directly from an Ethereum node, we need to iterate over all blocks from the inputted block number to the current block number. This is because there is no native JSON RPC method to get all transactions from a given address.

## Getting Started

You can run the application by cloning the repository and running the following command:

`HTTP_PROVIDER="HTTP_PROVIDER_URL" cargo run`

The `HTTP_PROVIDER_URL` is an URL to an Ethereum node, you can use Alchemy's free tier for this.

Alchemy has a generous free tier, but because this crawler runs concurrently through all blocks and transactions from the inputted block number to the current block number, it can take a while to run. If you want to run the crawler for a large number of blocks, you can use a paid tier from Alchemy or another Ethereum node provider.

To handle when the crawler exceeds the rate limit of the Ethereum node, the crawler uses `RetryClient` from `ethers-rs`. This allows the crawler to retry requests when the rate limit is exceeded using an exponential backoff strategy.

## Usage

When the server is started you can visit `localhost:8000` in your browser to view the application. You will be greeted with a form that allows you to input a wallet address and a block number. The application will then display the information about the transactions in a table with to, and from address and the amount of ether transferred.

## Tests

To run the tests for this application, run the following command:

`cargo test`
