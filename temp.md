```rust
use crate::{
    types::{CallTrace, LogTrace, StepTrace},
    utils::opcode_name,
};
use revm::{
    Inspector,
    context_interface::ContextTr,
    interpreter::{
        CallInputs, CallOutcome, Interpreter, interpreter::EthInterpreter, interpreter_types::Jumps,
    },
    primitives::Log,
};

#[derive(Debug, Default)]
pub struct MiniTracer {
    pub steps: Vec<StepTrace>,
    pub calls: Vec<CallTrace>,
    pub logs: Vec<LogTrace>,
    pub max_steps: Option<usize>,
    pub record_stack_top: usize,
    pub depth: usize,
    call_stack: Vec<usize>,
}

impl MiniTracer {
    pub fn new(max_steps: Option<usize>) -> Self {
        Self {
            max_steps,
            record_stack_top: 4,
            ..Default::default()
        }
    }

    pub fn should_record_step(&self) -> bool {
        self.max_steps
            .map(|max| self.steps.len() < max)
            .unwrap_or(true)
    }
}

impl<CTX> Inspector<CTX, EthInterpreter> for MiniTracer
where
    CTX: ContextTr,
{
    fn step(&mut self, interp: &mut Interpreter<EthInterpreter>, _context: &mut CTX) {
        if !self.should_record_step() {
            return;
        }

        let opcode = interp.bytecode.opcode();
        let stack_top = interp
            .stack
            .data()
            .iter()
            .rev()
            .take(self.record_stack_top)
            .map(|value| format!("{value:#x}"))
            .collect();

        self.steps.push(StepTrace {
            depth: self.current_frame_depth(),
            pc: interp.bytecode.pc(),
            opcode,
            opcode_hex: format!("0x{opcode:02x}"),
            opcode_name: opcode_name(opcode),
            gas_remaining: interp.gas.remaining(),
            stack_top,
            memory_size: interp.memory.len(),
        });
    }

    fn log(&mut self, _context: &mut CTX, log: Log) {
        self.logs.push(log_trace(&log));
    }

    fn call(&mut self, context: &mut CTX, inputs: &mut CallInputs) -> Option<CallOutcome> {
        let input = inputs.input.bytes(context);
        let call_index = self.calls.len();
        self.calls.push(CallTrace {
            depth: self.depth,
            kind: call_kind(inputs),
            from: inputs.caller.to_string(),
            to: inputs.target_address.to_string(),
            value: format!("{:#x}", inputs.call_value()),
            input: format!("0x{}", hex::encode(input)),
            gas_limit: inputs.gas_limit,
            success: None,
            gas_used: None,
        });
        self.call_stack.push(call_index);
        self.depth += 1;
        None
    }

    fn call_end(&mut self, _context: &mut CTX, _inputs: &CallInputs, outcome: &mut CallOutcome) {
        self.depth = self.depth.saturating_sub(1);

        if let Some(call_index) = self.call_stack.pop() {
            if let Some(call) = self.calls.get_mut(call_index) {
                call.success = Some(outcome.result.is_ok());
                call.gas_used = Some(outcome.result.gas.total_gas_spent());
            }
        }
    }
}

impl MiniTracer {
    fn current_frame_depth(&self) -> usize {
        self.depth.saturating_sub(1)
    }
}

fn call_kind(inputs: &CallInputs) -> String {
    format!("{:?}", inputs.scheme).to_uppercase()
}

fn log_trace(log: &Log) -> LogTrace {
    LogTrace {
        address: log.address.to_string(),
        topics: log.data.topics().iter().map(ToString::to_string).collect(),
        data: format!("0x{}", hex::encode(&log.data.data)),
    }
}

```

```rust
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
```

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
                storage,
            })
        })
        .collect::<Vec<_>>();

    diffs.sort_by(|left, right| left.address.cmp(&right.address));
    diffs
}

```

```rust
    fn call_end(&mut self, _context: &mut CTX, _inputs: &CallInputs, outcome: &mut CallOutcome) {
        self.depth = self.depth.saturating_sub(1);
        let success = outcome.result.is_ok();
        let gas_used = outcome.result.gas.total_gas_spent();

        if let Some(call_index) = self.call_stack.pop() {
            if let Some(call) = self.calls.get_mut(call_index) {
                call.success = Some(success);
                call.gas_used = Some(gas_used);
            }
        }

        if let Some(mut node) = self.call_tree_stack.pop() {
            node.success = Some(success);
            node.gas_used = Some(gas_used);

            if let Some(parent) = self.call_tree_stack.last_mut() {
                parent.children.push(node);
            } else {
                self.call_tree.push(node);
            }
        }
    }
```

```rust
fn build_call_tree(calls: &[CallTrace]) -> Vec<CallTreeNode> {
    let mut roots = Vec::new();
    let mut stack: Vec<CallTreeNode> = Vec::new();

    for call in calls {
        while stack.len() > call.depth {
            flush_call_tree_node(&mut stack, &mut roots);
        }

        stack.push(call_tree_node(call));
    }

    while !stack.is_empty() {
        flush_call_tree_node(&mut stack, &mut roots);
    }

    roots
}

fn flush_call_tree_node(stack: &mut Vec<CallTreeNode>, roots: &mut Vec<CallTreeNode>) {
    let Some(node) = stack.pop() else {
        return;
    };

    if let Some(parent) = stack.last_mut() {
        parent.children.push(node);
    } else {
        roots.push(node);
    }
}
```