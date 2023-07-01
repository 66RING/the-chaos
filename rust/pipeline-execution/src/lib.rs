#![feature(get_mut_unchecked)]

mod graph;
mod thread_pool;
mod pipeline;
mod processor;
mod source;
mod transform;

pub type Result<T> = std::result::Result<T, anyhow::Error>;

pub use graph::*;
pub use pipeline::*;
pub use processor::*;
pub use source::*;
pub use transform::*;


