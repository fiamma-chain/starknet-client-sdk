use starknet::{
    accounts::{Account, ExecutionEncoding, SingleOwnerAccount},
    core::{
        chain_id as starknet_chain_id,
        types::{Call, Felt},
        utils::{cairo_short_string_to_felt, get_selector_from_name},
    },
};

#[derive(Debug, Clone, Copy)]
pub enum StarknetChainId {
    Sepolia,
    Mainnet,
}

impl StarknetChainId {
    pub fn to_felt(&self) -> Felt {
        match self {
            Self::Sepolia => starknet_chain_id::SEPOLIA,
            Self::Mainnet => starknet_chain_id::MAINNET,
        }
    }
}
