use crate::{
    chain::StarknetChainId,
    types::{Peg, PegContext},
    utils::felt_to_u64,
};
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

pub struct BitveinBridgeClient {
    account: SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>,
    bitvm_bridge_contract: Felt,
    btc_light_client_contract: Felt,
}

impl BitveinBridgeClient {
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

    pub async fn mint_tokens(&self, ctx: &[PegContext]) -> anyhow::Result<String> {
        // Parse the inputs to Felt types
        let pegs = ctx
            .iter()
            .map(|ctx| Peg::try_from(ctx.clone()))
            .collect::<Result<Vec<Peg>, anyhow::Error>>()?;

        // Encode the calldata
        let mut calldata = vec![];
        pegs.encode(&mut calldata)?;

        // Execute the mint transaction
        let result = self
            .account
            .execute_v3(vec![Call {
                to: self.bitvm_bridge_contract,
                selector: get_selector_from_name("mint")
                    .map_err(|_| anyhow::anyhow!("Invalid mint selector"))?,
                calldata,
            }])
            .send()
            .await?;

        Ok(result.transaction_hash.to_hex_string())
    }

    pub async fn burn_tokens(
        &self,
        btc_address: &str,
        amount: u64,
        operator_id: u64,
    ) -> anyhow::Result<String> {
        // Parse the inputs to Felt types
        let btc_address = ByteArray::from(btc_address);
        let amount = Felt::from(amount);
        let operator_id = Felt::from(operator_id);

        // Encode the calldata
        let mut raw_calldata = vec![];
        btc_address.encode(&mut raw_calldata)?;
        raw_calldata.push(amount);
        raw_calldata.push(operator_id);

        // Execute the burn transaction
        let result = self
            .account
            .execute_v3(vec![Call {
                to: self.bitvm_bridge_contract,
                selector: get_selector_from_name("burn")
                    .map_err(|_| anyhow::anyhow!("Invalid burn selector"))?,
                calldata: raw_calldata,
            }])
            .send()
            .await?;

        Ok(result.transaction_hash.to_hex_string())
    }

    pub async fn query_latest_block_height(&self) -> anyhow::Result<u64> {
        let block_height = self
            .query_light_client_state(&FunctionCall {
                contract_address: self.btc_light_client_contract,
                entry_point_selector: get_selector_from_name("latest_block_height")
                    .map_err(|_| anyhow::anyhow!("Invalid latest_block_height selector"))?,
                calldata: vec![],
            })
            .await?;

        let block_height = block_height
            .first()
            .ok_or(anyhow::anyhow!("No block height found"))?;
        Ok(felt_to_u64(block_height)?)
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
        Ok(felt_to_u64(min_confirmations)?)
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
