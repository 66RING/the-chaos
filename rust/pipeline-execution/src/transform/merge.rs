use crate::{graph::ExGraph, processor::*, Result};
use std::{
    collections::VecDeque,
    fmt::Display,
    sync::{Arc, Mutex},
};

#[derive(Debug)]
pub struct MergeProcessor {
    name: &'static str,
    context: Arc<Context>,
    input: Vec<SharedDataPtr>,
    output: SharedDataPtr,
}

impl Display for MergeProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MergeProcessor [name: {}]", self.name)
    }
}

impl MergeProcessor {
    pub fn new(name: &'static str, graph: Arc<Mutex<ExGraph>>) -> Self {
        Self {
            name,
            context: Arc::new(Context::new(ProcessorType::Worker, graph)),
            input: Vec::new(),
            output: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
}

impl Processor for MergeProcessor {
    fn name(&self) -> &'static str {
        self.name
    }

    /// connect input from prev processor's output
    fn connect_from_input(&mut self, prev_processors: Vec<Arc<dyn Processor>>) {
        self.input = prev_processors.iter().map(|p| p.output_port()).collect()
    }

    /// wait until all prev processor finish and collect their output
    fn execute(&mut self) -> Result<()> {
        let finished = self
            .context()
            .get_prev_processors()
            .iter()
            .all(|x| x.context().get_state() == ProcessorState::Finished);
        if !finished {
            self.context().set_state(ProcessorState::Waiting);
            return Ok(());
        }

        // collect all output
        let mut output = self.output.lock().unwrap();
        for input in &self.input {
            let mut input = input.lock().unwrap();
            output.append(&mut input);
        }

        self.context().set_state(ProcessorState::Finished);
        self.set_next_processor_ready();
        Ok(())
    }

    fn output_port(&self) -> SharedDataPtr {
        self.output.clone()
    }

    fn context(&self) -> Arc<Context> {
        self.context.clone()
    }
}
