use crate::types::{CallTrace, LogTrace, StepTrace};

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