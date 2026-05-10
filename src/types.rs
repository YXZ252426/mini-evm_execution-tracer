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
    pub memory_preview_size: usize,
    pub memory_preview: String,
    pub memory_truncated: bool,
    pub storage: Vec<StepStorageTrace>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StepStorageTrace {
    pub op: String,
    pub address: String,
    pub slot: String,
    pub value_before: Option<String>,
    pub value_after: Option<String>,
    pub write_value: Option<String>,
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
pub struct CallTreeNode {
    pub depth: usize,
    pub kind: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub input: String,
    pub gas_limit: u64,
    pub success: Option<bool>,
    pub gas_used: Option<u64>,
    pub children: Vec<CallTreeNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogTrace {
    pub address: String,
    pub topics: Vec<String>,
    pub data: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct StorageDiff {
    pub slot: String,
    pub before: String,
    pub after: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StateDiff {
    pub address: String,
    pub storage: Vec<StorageDiff>,
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
    pub call_tree: Vec<CallTreeNode>,
    pub logs: Vec<LogTrace>,
    pub state_diff: Vec<StateDiff>,
}
