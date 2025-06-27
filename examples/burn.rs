use starknet_client_sdk::{bridge_client::BitvmBridgeClient, chain::StarknetChainId};

#[tokio::main]
async fn main() {
    let bridge_client = BitvmBridgeClient::new(
        "https://starknet-sepolia.public.blastapi.io/rpc/v0_8",
        "0x37f3357511947cc872aad08b97c49986b90479053630bffb8eeb968b757d255",
        "0x2a8812cad5c0b3ba20b97a0e519a6a5363849881958d2529a1e5313e55233cd",
        "0x293a3005233337f890c576e5c2768a47595f4cdbabd006c9898ce38a961fe7a",
        "0x0072b128ce0273e453e21b2d96a94bc72f5c297fcddae1a537f17769b4aaea80",
        &StarknetChainId::Sepolia,
    );

    let tx_hash = bridge_client
        .burn_tokens(
            "bcrt1phcnl4zcl2fu047pv4wx6y058v8u0n02at6lthvm7pcf2wrvjm5tqatn90k",
            5,
            500000,
            1,
        )
        .await
        .unwrap();

    println!("tx_hash: {}", tx_hash);

    let latest_block_height = bridge_client.query_latest_block_height().await.unwrap();
    println!("latest_block_height: {}", latest_block_height);

    let min_confirmations = bridge_client.query_min_confirmations().await.unwrap();
    println!("min_confirmations: {}", min_confirmations);
}
