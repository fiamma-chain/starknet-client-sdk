use serde::{Deserialize, Serialize};
use starknet::{
    accounts::{Account, ExecutionEncoding, SingleOwnerAccount},
    core::{
        codec::Encode,
        types::{ByteArray, Call, Felt, U256},
        utils::get_selector_from_name,
    },
    providers::{
        Url,
        jsonrpc::{HttpTransport, JsonRpcClient},
    },
    signers::{LocalWallet, SigningKey},
};

#[derive(Debug, Clone)]
pub struct PegContext {
    pub to: String,
    pub amount: u64,
    pub block_height: u64,
    // pub block_header: Vec<u8>,
    pub bitcoin_tx_hash: [u8; 32],
    pub bitcoin_tx_index: u32,
    // pub bitcoin_raw_tx: Vec<u8>,
    // pub bitcoin_merkle_proof: Vec<[u8; 32]>,
    pub output_index: u32,
    pub dest_script_hash: [u8; 32],
}

#[derive(Debug, Encode)]
pub(crate) struct BtcTxProof {
    pub tx_id: U256,
    pub tx_index: U256,
    // pub block_hash: U256,
}

#[derive(Debug, Encode)]
pub(crate) struct Peg {
    pub to: Felt,
    pub value: Felt,
    pub block_number: Felt,
    pub inclusion_proof: BtcTxProof,
    pub tx_out_ix: Felt,
    pub dest_script_hash: U256,
}

impl TryFrom<PegContext> for Peg {
    type Error = anyhow::Error;

    fn try_from(ctx: PegContext) -> Result<Self, Self::Error> {
        let to = Felt::from_hex(&ctx.to)?;
        let value = Felt::from(ctx.amount);
        let block_number = Felt::from(ctx.block_height);
        let inclusion_proof = BtcTxProof {
            tx_id: U256::from(crypto_bigint::U256::from_be_slice(&ctx.bitcoin_tx_hash)),
            tx_index: U256::from(ctx.bitcoin_tx_index),
        };
        let tx_out_ix = Felt::from(ctx.output_index);
        let dest_script_hash =
            U256::from(crypto_bigint::U256::from_be_slice(&ctx.dest_script_hash));
        Ok(Peg {
            to,
            value,
            block_number,
            inclusion_proof,
            tx_out_ix,
            dest_script_hash,
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
