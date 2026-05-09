use crate::{types::{CallTrace, LogTrace, StepTrace}, utils::opcode_name};
use revm::{
    Inspector, 
    context::ContextTr, 
    interpreter::{
        CallInputs, CallOutcome, Interpreter, interpreter::EthInterpreter, interpreter_types::Jumps
    }, primitives::Log
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
            record_stack_top: 6,
            ..Default::default()
        }
    }

    pub fn should_record_step(&self) -> bool {
        self.max_steps
            .map(|max| self.steps.len() < max)
            .unwrap_or(true)
    }
}

impl MiniTracer {
    fn current_frame_depth(&self) -> usize {
        self.depth.saturating_sub(1)
    }
}
impl<CTX> Inspector<CTX, EthInterpreter> for MiniTracer
where 
    CTX: ContextTr
{
    fn step(&mut self,interp: &mut Interpreter<EthInterpreter> , context: &mut CTX) {
        if !self.should_record_step() {
            return
        }

        let opcode = interp.bytecode.opcode();
        let stack_top = interp
            .stack
            .data()
            .iter()
            .rev()
            .take(self.record_stack_top)
            .map(| value | format!("{value:#x}"))
            .collect();

        self.steps.push(StepTrace { 
            depth: self.current_frame_depth(), 
            pc: interp.bytecode.pc(), 
            opcode, 
            opcode_hex: format!("0x{opcode:02x}"), 
            opcode_name: opcode_name(opcode), 
            gas_remaining: interp.gas.remaining(), 
            stack_top, 
            memory_size: interp.memory.len() 
        });
    }

    fn log(&mut self,context: &mut CTX,log: Log) {
        self.logs.push(log_trace(&log));
    }

    fn call(&mut self,context: &mut CTX,inputs: &mut CallInputs) -> Option<CallOutcome> {
        let input = inputs.input.bytes(context);

        self.calls.push(CallTrace { 
            depth: self.depth, 
            kind: call_kind(inputs), 
            from: inputs.transfer_from().to_string(), 
            to: inputs.transfer_to().to_string(), 
            value: format!("{:#x}", inputs.call_value()), 
            input: format!("0x{}", hex::encode(input)), 
            gas_limit: inputs.gas_limit, 
            success: None, 
            gas_used: None, 
        });
        None
    }

    fn call_end(&mut self,context: &mut CTX,inputs: &CallInputs,outcome: &mut CallOutcome) {
        self.depth = self.depth.saturating_sub(1);
        if let Some(call_index) = self.call_stack.pop() {
            if let Some(call) = self.calls.get_mut(call_index) {
                call.success = Some(outcome.result.is_ok());
                call.gas_used = Some(outcome.result.gas.total_gas_spent())
            }
        }
    }
}

fn call_kind(inputs: &CallInputs) -> String {
    format!("{:?}", inputs.scheme).to_uppercase()
}
fn log_trace(log: &Log ) -> LogTrace {
    LogTrace { 
        address: log.address.to_string(), 
        topics: log.data.topics().iter().map(ToString::to_string).collect(), 
        data:  format!("0x{}", hex::encode(&log.data.data)),
    }
}