use starknet::core::{chain_id as starknet_chain_id, types::Felt};

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
