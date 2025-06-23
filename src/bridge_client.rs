use crate::{
    chain::StarknetChainId,
    types::{BURN_FUNCTION_SELECTOR, ExecutionResult, MINT_FUNCTION_SELECTOR, Peg, PegContext},
    utils::felt_to_u64,
};
use anyhow::Ok;
use starknet::{
    accounts::{Account, ConnectedAccount, ExecutionEncoding, SingleOwnerAccount},
    core::{
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

pub struct BitvmBridgeClient {
    account: SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>,
    bitvm_bridge_contract: Felt,
    btc_light_client_contract: Felt,
}

impl BitvmBridgeClient {
    pub fn new(
        url: &str,
        bitvm_bridge_contract: &str,
        btc_light_client_contract: &str,
        private_key: &str,
        account_contract_address: &str,
        chain_id: &StarknetChainId,
    ) -> Self {
        let provider =
            JsonRpcClient::new(HttpTransport::new(Url::parse(url).expect("Invalid URL")));

        let signer = LocalWallet::from(SigningKey::from_secret_scalar(
            Felt::from_hex(private_key).expect("Invalid Starknet private key"),
        ));

        let account_contract_address = Felt::from_hex(account_contract_address)
            .expect("Invalid user account contract address");

        let bitvm_bridge_contract =
            Felt::from_hex(bitvm_bridge_contract).expect("Invalid bitvm bridge contract address");

        let btc_light_client_contract = Felt::from_hex(btc_light_client_contract)
            .expect("Invalid btc light client contract address");

        let account = SingleOwnerAccount::new(
            provider,
            signer,
            account_contract_address,
            chain_id.to_felt(),
            ExecutionEncoding::New,
        );
        Self {
            account,
            bitvm_bridge_contract,
            btc_light_client_contract,
        }
    }

    pub async fn mint_tokens(&self, contexts: &[PegContext]) -> anyhow::Result<String> {
        // Convert contexts to Peg structs
        let pegs: Result<Vec<Peg>, _> = contexts
            .iter()
            .map(|ctx| Peg::try_from(ctx.clone()))
            .collect();
        let pegs = pegs?;

        // Encode the calldata
        let mut calldata = vec![];
        pegs.encode(&mut calldata)?;

        // Execute the mint transaction
        let result = self
            .account
            .execute_v3(vec![Call {
                to: self.bitvm_bridge_contract,
                selector: MINT_FUNCTION_SELECTOR,
                calldata,
            }])
            .send()
            .await?;

        Ok(result.transaction_hash.to_hex_string())
    }

    pub async fn burn_tokens(
        &self,
        btc_address: &str,
        fee_rate: u32,
        amount: u64,
        operator_id: u32,
    ) -> anyhow::Result<String> {
        // Encode the calldata
        let mut calldata = vec![];

        // Encode each parameter according to the contract ABI
        ByteArray::from(btc_address).encode(&mut calldata)?;
        fee_rate.encode(&mut calldata)?;
        amount.encode(&mut calldata)?;
        operator_id.encode(&mut calldata)?;

        // Execute the burn transaction
        let result = self
            .account
            .execute_v3(vec![Call {
                to: self.bitvm_bridge_contract,
                selector: BURN_FUNCTION_SELECTOR,
                calldata,
            }])
            .send()
            .await?;

        Ok(result.transaction_hash.to_hex_string())
    }

    pub async fn query_latest_block_height(&self) -> anyhow::Result<u64> {
        let block_height = self
            .query_light_client_state(&FunctionCall {
                contract_address: self.btc_light_client_contract,
                entry_point_selector: get_selector_from_name("get_latest_block_height")
                    .map_err(|_| anyhow::anyhow!("Invalid latest_block_height selector"))?,
                calldata: vec![],
            })
            .await?;

        let block_height = block_height
            .first()
            .ok_or(anyhow::anyhow!("No block height found"))?;
        felt_to_u64(block_height)
    }

    pub async fn query_min_confirmations(&self) -> anyhow::Result<u64> {
        let min_confirmations = self
            .query_light_client_state(&FunctionCall {
                contract_address: self.btc_light_client_contract,
                entry_point_selector: get_selector_from_name("min_confirmations")
                    .map_err(|_| anyhow::anyhow!("Invalid min_confirmations selector"))?,
                calldata: vec![],
            })
            .await?;

        let min_confirmations = min_confirmations
            .first()
            .ok_or(anyhow::anyhow!("No min confirmations found"))?;
        felt_to_u64(min_confirmations)
    }

    pub async fn get_transaction_receipt(&self, tx_hash: &str) -> anyhow::Result<ExecutionResult> {
        let tx_hash = Felt::from_hex(tx_hash)?;
        let exe_res = self
            .account
            .provider()
            .get_transaction_receipt(tx_hash)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get transaction receipt with error: {:?}", e))?
            .receipt
            .execution_result()
            .clone();
        Ok(exe_res)
    }

    async fn query_light_client_state(&self, fc: &FunctionCall) -> anyhow::Result<Vec<Felt>> {
        let state = self
            .account
            .provider()
            .call(fc, BlockId::Tag(BlockTag::Latest))
            .await?;
        Ok(state)
    }
}
