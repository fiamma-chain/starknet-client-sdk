use async_trait::async_trait;
use starknet_client_sdk::events::{EventHandler, EventMonitor};

struct MyEventHandler;

#[async_trait]
impl EventHandler for MyEventHandler {
    async fn handle_mint(
        &self,
        block_number: u64,
        block_timestamp: u64,
        tx_hash: &str,
        to: &str,
        value: u64,
    ) -> anyhow::Result<()> {
        println!("Mint event detected:");
        println!("  Block number: {}", block_number);
        println!("  Block Timestamp: {}", block_timestamp);
        println!("  Transaction hash: {}", tx_hash);
        println!("  To: {}", to);
        println!("  Amount: {}", value);
        Ok(())
    }

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
    ) -> anyhow::Result<()> {
        println!("Burn event detected:");
        println!("  Block number: {}", block_number);
        println!("  Block Timestamp: {}", block_timestamp);
        println!("  Transaction hash: {}", tx_hash);
        println!("  From: {}", from);
        println!("  BTC Address: {}", btc_addr);
        println!("  Amount: {}", value);
        println!("  Fee rate: {}", fee_rate);
        println!("  Operator ID: {}", operator_id);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let url = "https://starknet-sepolia.public.blastapi.io/rpc/v0_8";
    let bitvm_bridge_contract_address =
        "0x014331d2b5b4f9a083941f5c45c402898b76f171343ed3ee4ae38f3a8c08b67b";

    // Initialize event handler
    let handler = Box::new(MyEventHandler);

    // Create and start event monitor
    let mut monitor = EventMonitor::new(bitvm_bridge_contract_address, url, handler, 880114);

    println!("Starting event monitor...");

    loop {
        let initial_processed_height = monitor.processed_height();

        match monitor.process().await {
            Ok(()) => {
                let latest_block_number = monitor.latest_block_number().await?;
                let processed_height = monitor.processed_height();

                println!(
                    "latest_block_number: {}, processed_height: {}",
                    latest_block_number, processed_height
                );

                // Check if we've caught up with the latest blocks
                if latest_block_number <= processed_height + 6 {
                    // 6 is CONFIRMED_BLOCKS
                    println!("Caught up with latest blocks. Monitoring for new blocks...");

                    // If no progress was made, sleep longer to avoid busy waiting
                    if processed_height == initial_processed_height {
                        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                    } else {
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    }
                } else {
                    // Still catching up, process immediately
                    println!("Still catching up, processing next batch...");
                }
            }
            Err(e) => {
                eprintln!("Error processing events: {}", e);
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }
}
