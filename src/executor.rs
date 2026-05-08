use crate::{
    tracer::MiniTracer,
    types::{TraceOutput, TraceSummary},
    utils::{parse_hex, read_hex_file},
};
use eyre::{Context as EyreContext, Ok, Result};
use revm::{
    Context, InspectEvm, MainBuilder, MainContext, 
    database::{CacheDB, EmptyDB},
    primitives::{Address, Bytes, TxKind, U256},
    state::{AccountInfo, Bytecode},
};
use std::path::Path;

pub fn trace_local(
    contract_path: &Path,
    call_data: &str,
    from: &str,
    to: &str,
    value: &str,
    gas_limit: u64,
    max_steps: Option<usize>,
) -> Result<TraceOutput> {
    // parse input
    let contract_bytes = read_hex_file(contract_path).wrap_err_with(|| {
        format!(
            "failed to read contract bytecode from {}",
            contract_path.display()
        )
    })?;
    let call_data = parse_hex(call_data).wrap_err("failed to parse calldata hex")?;
    let from = parse_address(from).wrap_err("failed to parse --from address")?;
    let to = parse_address(to).wrap_err("failed to parse --to address")?;
    let value = parse_u256(value).wrap_err("failed to parse --value")?;

    // build evm instance
    let mut db = CacheDB::new(EmptyDB::new());
    db.insert_account_info(from, AccountInfo::default().with_balance(U256::MAX));
    db.insert_account_info(
        to,
        AccountInfo::default().with_code(Bytecode::new_legacy(Bytes::from(contract_bytes)))
    );
    
    let tx = revm::context::TxEnv::builder()
        .caller(from)
        .kind(TxKind::Call(to))
        .value(value)
        .data(Bytes::from(call_data))
        .gas_limit(gas_limit)
        .build_fill();

    let tracer = MiniTracer::new(max_steps);
    let ctx = Context::mainnet()
        .modify_cfg_chained(|cfg| cfg.tx_gas_limit_cap = Some(u64::MAX))
        .with_db(db);
    let mut evm = ctx.build_mainnet_with_inspector(tracer);

    // deal with result
    let result_and_state = evm.inspect_tx(tx).wrap_err("revm execution failed")?;

    let tracer = evm.into_inspector();

    let success = result_and_state.result.is_success();
    let gas_used = result_and_state.result.tx_gas_used();

    Ok(TraceOutput { 
        summary: TraceSummary { 
            success,
            gas_used, 
            step_count: tracer.steps.len(), 
            call_count: tracer.calls.len(),
            log_count: tracer.logs.len(),
        },
        steps: tracer.steps,
        calls: tracer.calls,
        logs: tracer.logs,
    })
}

fn parse_address(input: &str) -> Result<Address> {
    input.parse::<Address>().map_err(Into::into)
} 

fn parse_u256(input: &str) -> Result<U256> {
    let input = input.trim();
    if let Some(hex) = input.strip_prefix("0x") {
        Ok(U256::from_str_radix(hex, 16)?)
    } else {
        Ok(U256::from_str_radix(input, 10)?)
    }
}
