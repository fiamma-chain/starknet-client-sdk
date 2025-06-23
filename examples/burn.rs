use starknet_client_sdk::{bridge_client::BitvmBridgeClient, chain::StarknetChainId};

#[tokio::main]
async fn main() {
    let bridge_client = BitvmBridgeClient::new(
        "https://starknet-sepolia.public.blastapi.io/rpc/v0_8",
        "0x014331d2b5b4f9a083941f5c45c402898b76f171343ed3ee4ae38f3a8c08b67b",
        "0x171ade3549cf65d6cd67d84149e2edf241f0614298663f5555b0c6f1983d5d4",
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
}
