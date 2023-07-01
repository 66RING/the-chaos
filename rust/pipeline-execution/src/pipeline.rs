
use std::sync::{Arc, Mutex};

use crate::graph::Index;
use crate::graph::ExGraph;
use crate::processor::Processor;
use crate::processor::ProcessorState;
use crate::transform::AccumulateProcessor;
use crate::transform::Accumulator;
use crate::thread_pool::ThreadPool;

use crate::Result;
use arrow::record_batch::RecordBatch;
use petgraph::stable_graph::NodeIndex;

#[derive(Debug)]
pub struct Pipeline {
    /// DAG graph that guide the execution of pipelien
    pub graph: Arc<Mutex<ExGraph>>,
    /// thread pool of executor
    thread_pool: ThreadPool,
    /// index of node in each level
    level_ids: Vec<Vec<Index>>,
}

impl Pipeline {
    pub fn new(num_threads: usize) -> Self {
        Self {
            graph: Arc::new(Mutex::new(ExGraph::new())),
            thread_pool: ThreadPool::new(num_threads),
            level_ids: Vec::new(),
        }
    }

    pub fn execute(&mut self) -> Result<Vec<RecordBatch>>  {
        // iter all processor to find the ready one
        let all_processors = self.graph.lock().unwrap().get_all_nodes();
        // loop until all node finished
        loop {
            let mut finished_num = 0;
            // find ready task and push to thread pool
            for p in &all_processors {
                match p.context().get_state() {
                    ProcessorState::Ready => {
                        let mut p = p.clone();
                        p.context().set_state(ProcessorState::Running);
                        self.thread_pool.spawn(move || unsafe {
                            let x = Arc::get_mut_unchecked(&mut p);
                            x.execute().unwrap();
                        });
                    },
                    ProcessorState::Waiting | ProcessorState::Running => {},
                    ProcessorState::Finished => finished_num += 1,
                }
            }

            // finished if all tasks done
            if finished_num == all_processors.len() {
                break;
            }
        }

        // get result from the last processor
        let last_processor = self.graph.lock().unwrap().get_last_node().unwrap();
        let output = last_processor
                .output_port()
                .lock()
                .unwrap()
                .drain(..)
                .collect();

        Ok(output)
    }

    pub fn add_processor(&mut self, processor: Arc<dyn Processor>) -> Index {
        self.graph.lock().unwrap().add_node(processor)
    }

    pub fn connect_processor(&mut self, from: Index, to: Index) {
        self.graph.lock().unwrap().add_edge(from, to);
    }

    pub fn add_source(&mut self, processor: Arc<dyn Processor>) {
        assert!(self.level_ids.len() <= 1);

        processor.context().set_state(ProcessorState::Ready);

        let index = self.add_processor(processor.clone());
        processor.context().set_index(index);

        if self.level_ids.is_empty() {
            self.level_ids.push(vec![index]);
        } else {
            self.level_ids[0].push(index);
        }
    }

    pub fn add_transform(&mut self, f: impl Fn(Arc<Mutex<ExGraph>>) -> Arc<dyn Processor>)  {
        assert!(!self.level_ids.is_empty());

        // get last level of pipeline
        let last_ids = self.level_ids.last().unwrap().clone();
        let mut transform_ids = vec![];
        for pipe_id in last_ids {
            // create processor for each branch
            let mut processor = f(self.graph.clone());
            let index = self.add_processor(processor.clone());
            processor.context().set_index(index);
            unsafe {
                let x = Arc::get_mut_unchecked(&mut processor);
                // dst connect to src node
                x.connect_from_input(vec![self
                    .graph
                    .lock()
                    .unwrap()
                    .get_node_by_index(pipe_id)]);
            }

            // connect to the tail of each branch
            self.connect_processor(pipe_id, index);
            transform_ids.push(index);
        }

        self.level_ids.push(transform_ids);
    }

    pub fn merge_processor(&mut self, accumulator: Accumulator, column_index: Option<usize>) {
        assert!(!self.level_ids.is_empty());

        // get last level
        let last_ids = self.level_ids.last().unwrap().clone();
        let mut acc_processor = Arc::new(AccumulateProcessor::new(
            "acc_processor",
            accumulator,
            column_index,
            self.graph.clone(),
        ));

        let acc_processor_index = self.add_processor(acc_processor.clone());
        acc_processor.context().set_index(acc_processor_index);
        let mut prev_processors = vec![];

        // connect all branch to this merge processor
        for index in last_ids {
            self.connect_processor(index, acc_processor_index);
            prev_processors.push(self.graph.lock().unwrap().get_node_by_index(index));
        }

        // connect to prev processors
        unsafe {
            let acc_processor = Arc::get_mut_unchecked(&mut acc_processor);
            acc_processor.connect_from_input(prev_processors);
        }
    }
}
