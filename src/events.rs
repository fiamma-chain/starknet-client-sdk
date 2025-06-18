use crate::{
    types::{TEST_EVENT_SELECTOR, TransactionEvent},
    utils::{block_timestamp, parse_event},
};
use async_trait::async_trait;
use starknet::{
    core::types::{BlockId, EventFilter, Felt},
    providers::{
        Provider, Url,
        jsonrpc::{HttpTransport, JsonRpcClient},
    },
};

/// Event handler trait for processing bridge events
#[async_trait]
pub trait EventHandler: Send + Sync {
    #[allow(clippy::too_many_arguments)]
    async fn handle_mint(
        &self,
        block_number: u64,
        block_timestamp: u64,
        tx_hash: &str,
        to: &str,
        value: u64,
    ) -> anyhow::Result<()>;

    #[allow(clippy::too_many_arguments)]
    async fn handle_burn(
        &self,
        block_number: u64,
        block_timestamp: u64,
        tx_hash: &str,
        from: &str,
        btc_addr: &str,
        value: u64,
        fee_rate: u64,
        operator_id: u64,
    ) -> anyhow::Result<()>;
}

/// Monitor for bridge events
pub struct EventMonitor {
    contract_address: Felt,
    handler: Box<dyn EventHandler>,
    provider: JsonRpcClient<HttpTransport>,
    last_processed_height: u64,
}

impl EventMonitor {
    const CHUNK_SIZE: u64 = 100;

    pub fn new(
        contract_address: &str,
        rpc_url: &str,
        handler: Box<dyn EventHandler>,
        last_processed_height: u64,
    ) -> Self {
        let provider = JsonRpcClient::new(HttpTransport::new(
            Url::parse(rpc_url).expect("Invalid starknet RPC URL"),
        ));
        let contract_address =
            Felt::from_hex(contract_address).expect("Invalid starknet contract address");
        Self {
            contract_address,
            handler,
            provider,
            last_processed_height,
        }
    }

    pub async fn process(&mut self) -> anyhow::Result<()> {
        let latest_block_number = self.latest_block_number().await?;

        if latest_block_number > self.last_processed_height {
            let next_height = self.last_processed_height + 1;

            // The events are paginated, so we need to use a continuation token to get the next chunk
            let mut continuation_token = None;
            loop {
                let response = self
                    .provider
                    .get_events(
                        EventFilter {
                            from_block: Some(BlockId::Number(next_height)),
                            to_block: Some(BlockId::Number(next_height)),
                            address: Some(self.contract_address),
                            // keys: Some(vec![vec![MINT_EVENT_SELECTOR, BURN_EVENT_SELECTOR]]),
                            keys: Some(vec![vec![TEST_EVENT_SELECTOR]]),
                        },
                        continuation_token,
                        Self::CHUNK_SIZE,
                    )
                    .await?;

                // Get block timestamp from the block
                let block_timestamp = if !response.events.is_empty() {
                    let block_with_txs = self
                        .provider
                        .get_block_with_tx_hashes(BlockId::Number(next_height))
                        .await?;
                    block_timestamp(&block_with_txs)
                } else {
                    0
                };

                // Process events here
                for event in response.events {
                    let block_number = event.block_number.unwrap_or(0);
                    let tx_hash = format!("0x{:x}", event.transaction_hash);
                    println!(
                        "block_number: {}, block_timestamp: {}, tx_hash: {}",
                        block_number, block_timestamp, tx_hash
                    );
                    if let Ok(parsed_event) = parse_event(&event) {
                        match parsed_event {
                            TransactionEvent::Test(test_event) => {
                                println!("test_event: {:?}", test_event);
                            }
                            TransactionEvent::Mint(mint_event) => {
                                self.handler
                                    .handle_mint(
                                        block_number,
                                        block_timestamp,
                                        &tx_hash,
                                        &mint_event.to,
                                        mint_event.value,
                                    )
                                    .await?;
                            }
                            TransactionEvent::Burn(burn_event) => {
                                self.handler
                                    .handle_burn(
                                        block_number,
                                        block_timestamp,
                                        &tx_hash,
                                        &burn_event.from,
                                        &burn_event.btc_addr,
                                        burn_event.value,
                                        burn_event.fee_rate as u64,
                                        burn_event.operator_id as u64,
                                    )
                                    .await?;
                            }
                        }
                    }
                }

                // Update continuation token for next iteration
                continuation_token = response.continuation_token;

                // If no continuation token, we've got all events
                if continuation_token.is_none() {
                    break;
                }
            }
            self.last_processed_height = next_height;
        }
        Ok(())
    }

    pub fn processed_height(&self) -> u64 {
        self.last_processed_height
    }

    pub async fn latest_block_number(&self) -> anyhow::Result<u64> {
        self.provider
            .block_number()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get latest block number: {}", e))
    }
}
