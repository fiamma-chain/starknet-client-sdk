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
        "0x03ad261eb4a1ee0dfae03d776e15e6e7110ad8e22e85c3f51c6ae4943aae474f";

    // Initialize event handler
    let handler = Box::new(MyEventHandler);

    // Create and start event monitor
    let mut monitor = EventMonitor::new(bitvm_bridge_contract_address, url, handler, 866565);
    loop {
        monitor.process().await?;

        let latest_block_number = monitor.latest_block_number().await?;
        let processed_height = monitor.processed_height();
        println!(
            "latest_block_number: {}, processed_height: {}",
            latest_block_number, processed_height
        );

        if latest_block_number == processed_height {
            break;
        }
    }

    Ok(())
}
