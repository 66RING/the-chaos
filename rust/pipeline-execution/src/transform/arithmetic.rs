use arrow::array::Array;
use arrow::array::Int32Array;

use arrow::datatypes::DataType;
use arrow::datatypes::Field;
use arrow::datatypes::Schema;
use arrow::record_batch::RecordBatch;

use crate::graph::ExGraph;
use crate::processor::*;
use crate::Result;
use std::collections::VecDeque;
use std::fmt::Display;
use std::sync::Arc;
use std::sync::Mutex;


#[derive(Debug, Clone, Copy)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug)]
pub struct ArithmeticTransform {
    name: &'static str,
    operator: Operator,
    context: Arc<Context>,
    /// index of left argument
    l_index: usize,
    /// index of right argument
    r_index: usize,
    input: SharedDataPtr,
    output: SharedDataPtr,
}

impl ArithmeticTransform {
    pub fn new(name: &'static str, graph: Arc<Mutex<ExGraph>>, operator: Operator, l_index: usize, r_index: usize) -> Self {
        Self {
            name,
            operator,
            context: Arc::new(Context::new(ProcessorType::Worker, graph)),
            l_index,
            r_index,
            input: Arc::new(Mutex::new(VecDeque::new())),
            output: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
}

impl Display for ArithmeticTransform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ArithmeticTransform [name: {}]", self.name)
    }
}

impl Processor for ArithmeticTransform {
    fn name(&self) -> &'static str {
        self.name
    }

    fn connect_from_input(&mut self, prev_processor: Vec<Arc<dyn Processor>>) {
        assert_eq!(prev_processor.len(), 1);
        self.input = prev_processor[0].output_port();
    }

    fn execute(&mut self) -> Result<()> {
        assert_eq!(self.context().get_prev_processors().len(), 1);

        let mut input = self.input.lock().unwrap();
        // fetch and pop all data from input
        for rb in input.drain(..) {
            let l_column = rb.column(self.l_index);
            let r_column = rb.column(self.r_index);

            // handle batch data and generate new batch data
            // for simple, we just do i32 data type
            let new_column = match rb.schema().field(self.l_index).data_type() {
                DataType::Int32 => {
                    let l_array = l_column.as_any().downcast_ref::<Int32Array>().unwrap();
                    let r_array = r_column.as_any().downcast_ref::<Int32Array>().unwrap();

                    let mut new_column_builder = Int32Array::builder(l_array.len());
                    for i in 0..l_array.len() {
                        let l_value = l_array.value(i);
                        let r_value = r_array.value(i);
                        let result = match self.operator {
                            Operator::Add => l_value + r_value,
                            Operator::Subtract => l_value - r_value,
                            Operator::Multiply => l_value * r_value,
                            Operator::Divide => l_value / r_value,
                        };
                        new_column_builder.append_value(result);
                    }
                    let new_column = new_column_builder.finish();
                    Arc::new(new_column)
                },
                _ => todo!(),
            };

            // construct new batch meta data: column and corresponding field
            let mut fields = vec![];
            let mut columns: Vec<Arc<dyn Array>> = vec![];

            let new_column_name = format!(
                "{} {} {}",
                rb.schema().field(self.l_index).name(),
                match self.operator {
                    Operator::Add => "+",
                    Operator::Subtract => "-",
                    Operator::Multiply => "*",
                    Operator::Divide => "/",
                },
                rb.schema().field(self.r_index).name()
            );
            let new_field = Field::new(&new_column_name, DataType::Int32, false);
            fields.push(new_field);
            columns.push(new_column);

            // copy the reset of data as it was
            for (i, column) in rb.columns().iter().enumerate() {
                if i == self.l_index || i == self.r_index {
                    continue;
                } else {
                    fields.push(rb.schema().field(i).clone());
                    columns.push(column.clone());
                }
            }

            let schema = Arc::new(Schema::new(fields));
            let mut output = self.output.lock().unwrap();
            output.push_back(RecordBatch::try_new(schema, columns)?);
        }

        // check whether the stream done
        if self.context().get_prev_processors()[0].context().get_state() == ProcessorState::Finished {
            self.context().set_state(ProcessorState::Finished);
        }

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




