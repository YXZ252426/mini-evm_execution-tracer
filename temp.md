```rust
use crate::{
    types::{CallTrace, LogTrace, StepTrace},
    utils::opcode_name,
};
use revm::{
    Inspector,
    interpreter::{Interpreter, interpreter::EthInterpreter, interpreter_types::Jumps},
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

impl<CTX> Inspector<CTX, EthInterpreter> for MiniTracer {
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
            depth: self.depth,
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
}

fn log_trace(log: &Log) -> LogTrace {
    LogTrace {
        address: log.address.to_string(),
        topics: log.data.topics().iter().map(ToString::to_string).collect(),
        data: format!("0x{}", hex::encode(&log.data.data)),
    }
}
```