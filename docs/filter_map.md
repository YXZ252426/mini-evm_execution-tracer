# `fliter_map`
a more flexible way of transform

very complex and nested logic, but also linear and natural to read if understand

```rust
fn collect_state_diff(state: &EvmState) -> Vec<StateDiff> {
    let mut diffs = state
        .iter()
        .filter_map(|(address, account)| {
            let mut storage = account
            .changed_storage_slots()
            .map(|(slot, value)| StorageDiff {
                slot: format!("{slot:#x}"),
                before: format!("{:#x}", value.original_value()),
                after: format!("{:#x}", value.present_value()),
            })
            .collect::<Vec<_>>();
            
            storage.sort_by(|left, right| left.slot.cmp(&right.slot));
            (!storage.is_empty()).then(|| StateDiff { 
                address: address.to_string(), 
                storage 
            })
        })
        .collect::<Vec<_>>();
    diffs.sort_by(|left, right| left.address.cmp(&right.address));
    diffs
}

```