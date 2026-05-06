use crate::types::{TraceOutput, TraceSummary};
use eyre::Result;
use std::path::Path;

pub fn trace_local(
    _contract_path: &Path,
    _call_data: &str,
    _from: &str,
    _to: &str,
    _value: &str,
    _gas_limit: u64,
    _max_steps: Option<usize>,
) -> Result<TraceOutput> {
    Ok(TraceOutput { 
        summary: TraceSummary { 
            success: true, 
            gas_used: 0, 
            step_count: 0, 
            call_count: 0, 
            log_count: 0 
        }, 
        steps: vec![], 
        calls: vec![], 
        logs: vec![]
    })
}