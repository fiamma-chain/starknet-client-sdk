use starknet::core::types::Felt;

// Make sure the felt is within u64 range
pub fn felt_to_u64(f: &Felt) -> anyhow::Result<u64> {
    let bytes = f.to_bytes_be();
    let value = u64::from_be_bytes(bytes[24..32].try_into()?);
    Ok(value)
}
