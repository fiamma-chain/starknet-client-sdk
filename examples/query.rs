use starknet_client_sdk::query_client::QueryClient;

#[tokio::main]
async fn main() {
    let query_client = QueryClient::new(
        "https://starknet-sepolia.g.alchemy.com/starknet/version/rpc/v0_8/xS1PQwOzOrX7U4AzG9IYnkgMWcdxQbX4",
    );

    let status = query_client
        .get_transaction_status("0x3387e2e2e6cff4d3e485e7c9343a7ec517c8098a6285f74a30956ecfa63be52")
        .await
        .unwrap();
    println!("status: {:?}", status);

    // let tx = query_client.get_transaction("0x260aa195b0f135083b6bfda8dbf65f457c7baf90bd628b090c28df5437ec302").await.unwrap();
    // println!("tx: {:?}", tx);

    let receipt = query_client
        .get_transaction_receipt(
            "0x260aa195b0f135083b6bfda8dbf65f457c7baf90bd628b090c28df5437ec302",
        )
        .await
        .unwrap();
    println!("receipt: {:?}", receipt.status());
}
