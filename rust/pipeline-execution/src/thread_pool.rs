//! ThreadPool wrapper of 

#[derive(Debug)]
pub struct ThreadPool(rayon::ThreadPool);

impl ThreadPool {
    pub fn new(num_threads: usize) -> Self {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .unwrap();
        ThreadPool(pool)
    }
    
    pub fn spawn<Op>(&self, op: Op) 
    where
        Op: FnOnce() + Send + 'static,
    {
        self.0.spawn(op)
    }
}
