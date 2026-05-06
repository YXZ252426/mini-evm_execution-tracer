use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct StepTrace {
    pub depth: usize,
    pub pc: usize,
    pub opcode: u8,
    pub opcode_hex: String,
    pub opcode_name: String,
    pub gas_remaining: u64,
    pub stack_top: Vec<String>,
    pub memory_size: usize,
}
#[derive(Debug, Clone, Serialize)]
pub struct CallTrace {
    pub depth: usize,
    pub kind: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub input: String,
    pub gas_limit: u64,
    pub success: Option<bool>,
    pub gas_used: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogTrace {
    pub address: String,
    pub topics: Vec<String>,
    pub data: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TraceSummary {
    pub success: bool,
    pub gas_used: u64,
    pub step_count: usize,
    pub call_count: usize,
    pub log_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct TraceOutput {
    pub summary: TraceSummary,
    pub steps: Vec<StepTrace>,
    pub calls: Vec<CallTrace>,
    pub logs: Vec<LogTrace>,
}

