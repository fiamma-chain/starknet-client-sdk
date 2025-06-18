use crate::types::{
    BURN_EVENT_SELECTOR, BurnEventData, BurnEventDataWithoutKey, MINT_EVENT_SELECTOR, MintEventData,
    TEST_EVENT_SELECTOR, TestEventData, TestEventDataWithoutKey, TransactionEvent,
};
use starknet::core::{
    codec::Decode,
    types::{EmittedEvent, Felt, MaybePendingBlockWithTxHashes},
    utils::parse_cairo_short_string,
};

// Make sure the felt is within u64 range
pub fn felt_to_u64(f: &Felt) -> anyhow::Result<u64> {
    let bytes = f.to_bytes_be();
    let value = u64::from_be_bytes(bytes[24..32].try_into()?);
    Ok(value)
}

// First key is the selector, if second key is exists, it is the key(indexer) of the event data
pub fn parse_event(event: &EmittedEvent) -> anyhow::Result<TransactionEvent> {
    let key = event
        .keys
        .first()
        .ok_or(anyhow::anyhow!("Invalid event keys"))?;

    if *key == TEST_EVENT_SELECTOR {
        let alice = format!("{:x}", &event.keys[1]);
        let other = TestEventDataWithoutKey::decode(&event.data)?;
        let bob = String::try_from(other.bob)?;
        Ok(TransactionEvent::Test(TestEventData {
            alice,
            bob,
            value: other.amount,
        }))
    } else if *key == MINT_EVENT_SELECTOR {
        let to = parse_cairo_short_string(&event.keys[1])?;
        let value = felt_to_u64(
            event
                .data
                .last()
                .ok_or(anyhow::anyhow!("Invalid event data"))?,
        )?;
        Ok(TransactionEvent::Mint(MintEventData { to, value }))
    } else if *key == BURN_EVENT_SELECTOR {
        let from = parse_cairo_short_string(&event.keys[1])?;
        let other = BurnEventDataWithoutKey::decode(&event.data)?;
        Ok(TransactionEvent::Burn(BurnEventData::from_without_key(
            from, &other,
        )))
    } else {
        anyhow::bail!("Unspported event type")
    }
}

pub fn block_timestamp(block: &MaybePendingBlockWithTxHashes) -> u64 {
    match block {
        MaybePendingBlockWithTxHashes::Block(block) => block.timestamp,
        MaybePendingBlockWithTxHashes::PendingBlock(block) => block.timestamp,
    }
}

#[test]
fn test_event_keys() {
    use starknet::core::utils::get_selector_from_name;
    let selector = get_selector_from_name("increase_balance").unwrap();
    println!("selector: {:?}", selector);
}

#[test]
fn test_data() {
    let data = vec![
        Felt::from_hex("0x2").unwrap(),
        Felt::from_hex("0x62633170356437726a7137673672646b3279687a6b7339736d6c6171746564").unwrap(),
        Felt::from_hex("0x723464656b7130386765387a74776163373273667239727573786733323937").unwrap(),
        Felt::from_hex("0x0").unwrap(),
        Felt::from_hex("0x0").unwrap(),
        Felt::from_hex("0x64").unwrap(),
    ];

    let result = TestEventDataWithoutKey::decode(&data).unwrap();
    println!(
        "bob: {}, amount: {}",
        String::try_from(result.bob).unwrap(),
        result.amount
    );
}
