use crate::types::TraceOutput;
use eyre::Result;
use std::{fs, path::Path};

pub fn print_summary(output: &TraceOutput) {
    println!(
        "tx status: {}",
        if output.summary.success {
            "success"
        } else {
            "failed "
        }
    );
    println!("gas used: {}", output.summary.gas_used);
    println!("steps: {}", output.summary.step_count);
    println!("calls: {}", output.summary.call_count);
    println!("logs: {}", output.summary.log_count);
}

pub fn write_json(path: &Path, output: &TraceOutput) -> Result<()> {
    let json = serde_json::to_string_pretty(output)?;
    fs::write(path, json)?;
    Ok(())
}
