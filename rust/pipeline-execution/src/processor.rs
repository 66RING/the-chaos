//! basic trait infomation of processor and the state machine

use std::sync::{Mutex, Arc};
use std::collections::VecDeque;
use crate::graph::{Index, ExGraph};
use crate::Result;

use arrow::record_batch::RecordBatch;


pub type SharedData = Mutex<VecDeque<RecordBatch>>;
pub type SharedDataPtr = Arc<SharedData>;


#[derive(Clone, Copy, Debug)]
pub enum ProcessorType {
    Source,
    Worker,
    Sink,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProcessorState {
    Waiting,
    Running,
    Ready,
    Finished,
}

/// store meta data that a Processor trait may need
#[derive(Debug)]
pub struct Context {
    processor_type: ProcessorType,
    state: Mutex<ProcessorState>,
    index: Mutex<Index>,
    graph: Arc<Mutex<ExGraph>>,
}

impl Context {
    pub fn new(processor_type: ProcessorType, graph: Arc<Mutex<ExGraph>>) -> Self {
        Self {
            state: Mutex::new(ProcessorState::Waiting),
            processor_type,
            index: Mutex::new(Index::default()),
            graph,
        }
    }

    pub fn set_state(&self, new_state: ProcessorState) {
        let mut state = self.state.lock().unwrap();
        *state = new_state;
    }

    pub fn get_state(&self) -> ProcessorState {
        *self.state.lock().unwrap()
    }

    pub fn set_index(&self, new_index: Index) {
        let mut index = self.index.lock().unwrap();
        *index = new_index;
    }

    pub fn get_index(&self) -> Index {
        *self.index.lock().unwrap()
    }

    pub fn get_next_processors(&self) -> Vec<Arc<dyn Processor>> {
        self.graph.lock().unwrap().get_next_nodes(self.get_index())
    }

    pub fn get_prev_processors(&self) -> Vec<Arc<dyn Processor>> {
        self.graph.lock().unwrap().get_prev_nodes(self.get_index())
    }
}

pub trait Processor: Send + Sync + std::fmt::Debug + std::fmt::Display {
    fn name(&self) -> &'static str;

    fn connect_from_input(&mut self, input: Vec<Arc<dyn Processor>>);

    fn execute(&mut self) -> Result<()>;

    fn output_port(&self) -> SharedDataPtr;

    fn context(&self) -> Arc<Context>;

    fn set_next_processor_ready(&self) {
        let next_processors = self.context().get_next_processors();
        for processor in next_processors {
            processor.context().set_state(ProcessorState::Ready);
        }
    }
}
