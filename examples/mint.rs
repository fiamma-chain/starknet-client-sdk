use starknet::{
    accounts::{Account, ConnectedAccount, ExecutionEncoding, SingleOwnerAccount},
    core::{
        chain_id,
        codec::Encode,
        types::{BlockId, BlockTag, ByteArray, Call, Felt, FunctionCall},
        utils::get_selector_from_name,
    },
    providers::{
        Provider, Url,
        jsonrpc::{HttpTransport, JsonRpcClient},
    },
    signers::{LocalWallet, SigningKey},
};
use starknet_client_sdk::utils::felt_to_u64;

#[tokio::main]
async fn main() {
    let provider = JsonRpcClient::new(HttpTransport::new(
        Url::parse("https://rpc.starknet-testnet.lava.build:443").unwrap(),
    ));

    let signer = LocalWallet::from(SigningKey::from_secret_scalar(
        Felt::from_hex("0x293a3005233337f890c576e5c2768a47595f4cdbabd006c9898ce38a961fe7a")
            .unwrap(),
    ));
    let address =
        Felt::from_hex("0x0072b128ce0273e453e21b2d96a94bc72f5c297fcddae1a537f17769b4aaea80")
            .unwrap();
    let test_contract_address =
        Felt::from_hex("0x00af36cd621814b623bd6261c55d3b81d6369ca6d86f8c282f2f3f9053cb624f")
            .unwrap();

    let account = SingleOwnerAccount::new(
        provider,
        signer,
        address,
        chain_id::SEPOLIA,
        ExecutionEncoding::New,
    );

    let test_string = "xxxxxxxxxxxxxxxyyyyyyyyyyyyyyyyyyyyyyzzzzzzzzzzzzzzzzz";
    let test_string = ByteArray::from(test_string);
    let mut raw_test_string = vec![];
    test_string
        .encode(&mut raw_test_string)
        .expect("Failed to encode bytearray string");
    raw_test_string.push(Felt::from_dec_str("2000000").unwrap());

    let result = account
        .execute_v3(vec![Call {
            to: test_contract_address,
            selector: get_selector_from_name("increase_balance").unwrap(),
            calldata: raw_test_string,
        }])
        .send()
        .await
        .unwrap();

    println!("Transaction hash: {:#064x}", result.transaction_hash);

    let block_height = account.provider().block_number().await.unwrap();
    println!("Block height: {}", block_height);

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let balance = account
        .provider()
        .call(
            FunctionCall {
                contract_address: test_contract_address,
                entry_point_selector: get_selector_from_name("get_balance").unwrap(),
                calldata: vec![],
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await
        .unwrap();

    let balance = felt_to_u64(balance.first().unwrap()).unwrap();
    println!("Balance: {:?}", balance);
}
