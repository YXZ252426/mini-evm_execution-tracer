use crate::{
    types::{CallTrace, LogTrace, StepStorageTrace, StepTrace},
    utils::opcode_name,
};
use revm::{
    Inspector, JournalEntry,
    bytecode::opcode,
    context_interface::ContextTr,
    inspector::JournalExt,
    interpreter::{
        CallInputs, CallOutcome, Interpreter,
        interpreter::EthInterpreter,
        interpreter_types::{InputsTr, Jumps},
    },
    primitives::{Address, Log, U256},
};

#[derive(Debug, Default)]
pub struct MiniTracer {
    pub steps: Vec<StepTrace>,
    pub calls: Vec<CallTrace>,
    pub logs: Vec<LogTrace>,
    pub max_steps: Option<usize>,
    pub record_stack_top: usize,
    pub memory_preview_bytes: usize,
    pub depth: usize,
    call_stack: Vec<usize>,
    last_journal_len: usize,
    pending_storage_step: Option<PendingStorageStep>,
}

#[derive(Debug, Clone, Copy)]
struct PendingStorageStep {
    step_index: usize,
    address: Address,
    slot: U256,
}

impl MiniTracer {
    pub fn new(max_steps: Option<usize>) -> Self {
        Self {
            max_steps,
            record_stack_top: 6,
            memory_preview_bytes: 64,
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
    CTX: ContextTr<Journal: JournalExt>,
{
    fn step(&mut self, interp: &mut Interpreter<EthInterpreter>, context: &mut CTX) {
        if !self.should_record_step() {
            return;
        }

        let opcode = interp.bytecode.opcode();
        self.last_journal_len = context.journal_ref().journal().len();
        let stack_top = interp
            .stack
            .data()
            .iter()
            .rev()
            .take(self.record_stack_top)
            .map(|value| format!("{value:#x}"))
            .collect();
        let memory = memory_preview(interp, self.memory_preview_bytes);
        let step_index = self.steps.len();
        let (storage, pending_storage_step) = storage_for_step(interp, context, opcode)
            .map(|storage| {
                let pending = PendingStorageStep {
                    step_index,
                    address: interp.input.target_address(),
                    slot: storage.slot_raw,
                };
                (vec![storage.trace], Some(pending))
            })
            .unwrap_or_else(|| (Vec::new(), None));
        self.pending_storage_step = pending_storage_step;

        self.steps.push(StepTrace {
            depth: self.current_frame_depth(),
            pc: interp.bytecode.pc(),
            opcode,
            opcode_hex: format!("0x{opcode:02x}"),
            opcode_name: opcode_name(opcode),
            gas_remaining: interp.gas.remaining(),
            stack_top,
            memory_size: interp.memory.len(),
            memory_preview_size: memory.preview_size,
            memory_preview: memory.preview,
            memory_truncated: memory.truncated,
            storage,
        });
    }

    fn step_end(&mut self, _interp: &mut Interpreter<EthInterpreter>, context: &mut CTX) {
        let Some(pending) = self.pending_storage_step.take() else {
            return;
        };

        let journal = context.journal_ref().journal();
        let storage_change = if journal.len() != self.last_journal_len {
            journal
                .last()
                .and_then(|entry| storage_change(entry, context))
        } else {
            None
        };
        let fallback = storage_values(context, pending.address, pending.slot);

        let Some(step) = self.steps.get_mut(pending.step_index) else {
            return;
        };
        let Some(storage) = step.storage.first_mut() else {
            return;
        };

        if let Some(change) = storage_change
            && change.address == pending.address
            && change.slot == pending.slot
        {
            storage.value_before = Some(change.value_before);
            storage.value_after = Some(change.value_after);
            return;
        }

        if let Some(values) = fallback {
            storage
                .value_before
                .get_or_insert_with(|| values.present.clone());
            storage.value_after = Some(values.present);
        }
    }

    fn log(&mut self, _context: &mut CTX, log: Log) {
        self.logs.push(log_trace(&log));
    }

    fn call(&mut self, context: &mut CTX, inputs: &mut CallInputs) -> Option<CallOutcome> {
        let input = inputs.input.bytes(context);
        let call = CallTrace {
            depth: self.depth,
            kind: call_kind(inputs),
            from: inputs.caller.to_string(),
            to: inputs.target_address.to_string(),
            value: format!("{:#x}", inputs.call_value()),
            input: format!("0x{}", hex::encode(input)),
            gas_limit: inputs.gas_limit,
            success: None,
            gas_used: None,
        };

        let call_index = self.calls.len();
        self.calls.push(call);
        self.call_stack.push(call_index);
        self.depth += 1;
        None
    }

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

#[derive(Debug)]
struct MemoryPreview {
    preview_size: usize,
    preview: String,
    truncated: bool,
}

fn memory_preview(interp: &Interpreter<EthInterpreter>, max_bytes: usize) -> MemoryPreview {
    let memory = interp.memory.context_memory();
    let preview_size = memory.len().min(max_bytes);

    MemoryPreview {
        preview_size,
        preview: format!("0x{}", hex::encode(&memory[..preview_size])),
        truncated: memory.len() > max_bytes,
    }
}

#[derive(Debug)]
struct StorageForStep {
    trace: StepStorageTrace,
    slot_raw: U256,
}

fn storage_for_step<CTX>(
    interp: &Interpreter<EthInterpreter>,
    context: &CTX,
    opcode: u8,
) -> Option<StorageForStep>
where
    CTX: ContextTr<Journal: JournalExt>,
{
    match opcode {
        opcode::SLOAD => {
            let slot = interp.stack.peek(0).ok()?;
            let address = interp.input.target_address();
            let before = storage_values(context, address, slot).map(|values| values.present);

            Some(StorageForStep {
                trace: StepStorageTrace {
                    op: "SLOAD".to_string(),
                    address: address.to_string(),
                    slot: format!("{slot:#x}"),
                    value_before: before,
                    value_after: None,
                    write_value: None,
                },
                slot_raw: slot,
            })
        }
        opcode::SSTORE => {
            let slot = interp.stack.peek(0).ok()?;
            let write_value = interp.stack.peek(1).ok()?;
            let address = interp.input.target_address();
            let before = storage_values(context, address, slot).map(|values| values.present);

            Some(StorageForStep {
                trace: StepStorageTrace {
                    op: "SSTORE".to_string(),
                    address: address.to_string(),
                    slot: format!("{slot:#x}"),
                    value_before: before,
                    value_after: None,
                    write_value: Some(format!("{write_value:#x}")),
                },
                slot_raw: slot,
            })
        }
        _ => None,
    }
}

#[derive(Debug)]
struct StorageValues {
    original: String,
    present: String,
}

fn storage_values<CTX>(context: &CTX, address: Address, slot: U256) -> Option<StorageValues>
where
    CTX: ContextTr<Journal: JournalExt>,
{
    let slot = context
        .journal_ref()
        .evm_state()
        .get(&address)?
        .storage
        .get(&slot)?;

    Some(StorageValues {
        original: format!("{:#x}", slot.original_value()),
        present: format!("{:#x}", slot.present_value()),
    })
}

#[derive(Debug)]
struct StorageChange {
    address: Address,
    slot: U256,
    value_before: String,
    value_after: String,
}

fn storage_change<CTX>(entry: &JournalEntry, context: &CTX) -> Option<StorageChange>
where
    CTX: ContextTr<Journal: JournalExt>,
{
    match entry {
        JournalEntry::StorageChanged {
            address,
            key,
            had_value,
        } => {
            let value_after = context
                .journal_ref()
                .evm_state()
                .get(address)?
                .storage
                .get(key)?
                .present_value();

            Some(StorageChange {
                address: *address,
                slot: *key,
                value_before: format!("{had_value:#x}"),
                value_after: format!("{value_after:#x}"),
            })
        }
        JournalEntry::StorageWarmed { address, key } => {
            let values = storage_values(context, *address, *key)?;
            Some(StorageChange {
                address: *address,
                slot: *key,
                value_before: values.original,
                value_after: values.present,
            })
        }
        _ => None,
    }
}
