use crypto_bigint::Encoding;
use serde::{Deserialize, Serialize};
use starknet::core::{
    codec::{Decode, Encode},
    types::{ByteArray, Felt, U256},
};

#[allow(dead_code)]
pub use starknet::core::types::{
    ExecutionResult, Felt as StarknetAddress, Transaction, TransactionExecutionStatus,
    TransactionStatus,
};

// starknet_keccak("mint")
pub const MINT_FUNCTION_SELECTOR: Felt =
    Felt::from_hex_unchecked("0x2f0b3c5710379609eb5495f1ecd348cb28167711b73609fe565a72734550354");

// starknet_keccak("burn")
pub const BURN_FUNCTION_SELECTOR: Felt =
    Felt::from_hex_unchecked("0x3e8cfd4725c1e28fa4a6e3e468b4fcf75367166b850ac5f04e33ec843e82c1");

// starknet_keccak("Mint")
pub const MINT_EVENT_SELECTOR: Felt =
    Felt::from_hex_unchecked("0x34e55c1cd55f1338241b50d352f0e91c7e4ffad0e4271d64eb347589ebdfd16");

// starknet_keccak("Burn")
pub const BURN_EVENT_SELECTOR: Felt =
    Felt::from_hex_unchecked("0x243e1de00e8a6bc1dfa3e950e6ade24c52e4a25de4dee7fb5affe918ad1e744");

// starknet_keccak("BalanceIncreased")
pub const TEST_EVENT_SELECTOR: Felt =
    Felt::from_hex_unchecked("0x31f8daa2ac8dacd06ab968bad8f97f98f63c30a86dbfcebdd7625f589d4e7e6");

// TODO: this is only for test
#[derive(Debug, PartialEq, Eq, Decode)]
pub struct TestEventDataWithoutKey {
    pub bob: ByteArray,
    pub amount: u32,
}

#[derive(Debug)]
pub enum TransactionEvent {
    Mint(MintEventData),
    Burn(BurnEventData),
    Test(TestEventData), // TODO: this is only for test
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MintEventData {
    pub to: String,
    pub value: u64,
}

#[derive(Debug, PartialEq, Eq, Decode)]
pub struct BurnEventDataWithoutKey {
    pub btc_addr: ByteArray,
    pub fee_rate: u32,
    pub value: u64,
    pub operator_id: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BurnEventData {
    pub from: String,
    pub btc_addr: String,
    pub fee_rate: u32,
    pub value: u64,
    pub operator_id: u32,
}

impl BurnEventData {
    pub fn from_without_key(from: String, data: &BurnEventDataWithoutKey) -> Self {
        Self {
            from,
            btc_addr: String::try_from(data.btc_addr.clone()).unwrap(),
            fee_rate: data.fee_rate,
            value: data.value,
            operator_id: data.operator_id,
        }
    }
}

// TODO: this is only for test, remove this after mint & burn is ready
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestEventData {
    pub alice: String,
    pub bob: String,
    pub value: u32,
}

#[derive(Debug, Clone)]
pub struct PegContext {
    pub to: String,
    pub amount: u64,
    pub block_height: u64,
    pub block_header: Vec<u8>,
    pub bitcoin_tx_hash: [u8; 32],
    pub bitcoin_tx_index: u32,
    pub bitcoin_raw_tx: Vec<u8>,
    pub bitcoin_merkle_proof: Vec<[u8; 32]>,
    pub output_index: u32,
    pub dest_script_hash: [u8; 32],
}

#[derive(Debug, Encode)]
pub(crate) struct BtcTxProof {
    pub block_header: Vec<u8>,
    pub tx_id: U256,
    pub tx_index: u32,
    pub merkle_proof: Vec<U256>,
    pub raw_tx: Vec<u8>,
}

#[derive(Debug, Encode)]
pub(crate) struct Peg {
    pub to: Felt,
    pub value: u64,
    pub block_num: u32,
    pub inclusion_proof: BtcTxProof,
    pub tx_out_ix: u32,
    pub dest_script_hash: U256,
}

impl TryFrom<PegContext> for Peg {
    type Error = anyhow::Error;

    fn try_from(ctx: PegContext) -> Result<Self, Self::Error> {
        let to = Felt::from_hex(&ctx.to)?;

        // Convert merkle proof more efficiently using iterator
        let merkle_proof: Vec<U256> = ctx
            .bitcoin_merkle_proof
            .iter()
            .map(|&hash| U256::from(crypto_bigint::U256::from_le_bytes(hash)))
            .collect();

        let inclusion_proof = BtcTxProof {
            block_header: ctx.block_header,
            tx_id: U256::from(crypto_bigint::U256::from_le_slice(&ctx.bitcoin_tx_hash)),
            tx_index: ctx.bitcoin_tx_index,
            merkle_proof,
            raw_tx: ctx.bitcoin_raw_tx,
        };

        Ok(Peg {
            to,
            value: ctx.amount,
            block_num: ctx.block_height as u32,
            inclusion_proof,
            tx_out_ix: ctx.output_index,
            dest_script_hash: U256::from(crypto_bigint::U256::from_be_slice(&ctx.dest_script_hash)),
        })
    }
}

impl Peg {
    #[allow(dead_code)]
    pub fn to_calldata(&self) -> anyhow::Result<Vec<Felt>> {
        let mut encoded = vec![];
        self.encode(&mut encoded)?;
        Ok(encoded)
    }
}
