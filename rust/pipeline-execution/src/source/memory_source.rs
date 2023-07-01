use arrow::record_batch::RecordBatch;


use crate::graph::ExGraph;
use crate::processor::*;
use crate::Result;
use std::collections::VecDeque;
use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};


#[derive(Debug)]
pub struct MemorySource {
    context: Arc<Context>,

    data: Vec<RecordBatch>,
    /// index of current data
    index: usize,
    output: SharedDataPtr,
}

impl Display for MemorySource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MemorySource")
    }
}

impl MemorySource {
    pub fn new(data: Vec<RecordBatch>, graph: Arc<Mutex<ExGraph>>) -> Self {
        Self { 
            context: Arc::new(Context::new(ProcessorType::Source, graph)),
            data: data,
            index: 0,
            output: Arc::new(Mutex::new(VecDeque::new())), 
        }
    }
}

impl Processor for MemorySource {
    fn name(&self) -> &'static str {
        "MemorySource"
    }

    fn connect_from_input(&mut self, _: Vec<Arc<dyn Processor>>) {
        panic!("Source need no input")
    }

    /// Pass in a batch of input to execute
    fn execute(&mut self) -> Result<()> {
        // pass data to next processor
        if self.index < self.data.len() {
            let batch = self.data[self.index].clone();
            self.output.lock().unwrap().push_back(batch);
            self.index += 1;
        }

        if self.index == self.data.len() {
            self.context.set_state(ProcessorState::Finished);
        }
        // notify next processor
        self.set_next_processor_ready();
        Ok(())
    }

    /// Get output
    fn output_port(&self) -> SharedDataPtr {
        self.output.clone()
    }

    /// Get context
    fn context(&self) -> Arc<Context> {
        self.context.clone()
    }
}



