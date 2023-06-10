use std::mem;

/// cap = N
struct FixedHeap<T, const N: usize> {
    size: usize,
    data: [T; N],
}

impl<T, const N: usize> FixedHeap<T, N>
where
    T: Copy + Default,
{
    pub fn new() -> Self {
        Self {
            size: 0,
            data: [Default::default(); N],
        }
    }

    /// return None if have spare space
    /// return Some(T) if full, which eliminate the smallest value
    pub fn push<F, S>(&mut self, value: T, comparer: &F, state: &S) -> Option<T>
    where
        F: Fn(&T, &T, &S) -> bool,
    {
        let mut result = None;
        let mut node_index = self.size;
        if N == 0 {
            return Some(value);
        }

        if let Some(value) = self.add_last(value) {
            // if full, should eliminate smallest node.
            // add tail or add into the smallest index
            if self.size == N {
                // if full, should eliminate smallest value
                let mut smallest_index = N >> 1;
                // find the smallest node in the tree.
                for index in (N >> 1)..N {
                    let node_value = &self.data[index];
                    let smallest_value = &self.data[smallest_index];
                    if !comparer(node_value, smallest_value, state) {
                        // if node_value < smallest_value
                        smallest_index = index;
                    }
                }

                let smallest_value = &self.data[smallest_index];
                if comparer(&value, smallest_value, state) {
                    // replace the smallest one.
                    let replaced = mem::replace(
                        unsafe { self.data.get_unchecked_mut(smallest_index) },
                        value,
                    );

                    // update node index.
                    node_index = smallest_index;
                    result = Some(replaced);
                } else {
                    // if insert node smaller, return itself and do nothing.
                    return Some(value);
                }
            }
        }

        // swim up
        while node_index != 0 {
            let parent_index = (node_index - 1) >> 1;
            let node_value = &self.data[node_index];
            let parent_value = &self.data[parent_index];

            // if node > parent swim up
            if comparer(node_value, parent_value, state) {
                self.data.swap(node_index, parent_index);
                node_index = parent_index;
            } else {
                break;
            }
        }

        result
    }

    /// return highest value
    /// return None if empty
    pub fn pop<F, S>(&mut self, comparer: &F, state: &S) -> Option<T>
    where
        F: Fn(&T, &T, &S) -> bool,
    {
        self.pop_at(0, comparer, state)
    }

    /// return highest value
    /// return None if empty
    fn pop_at<F, S>(&mut self, index: usize, comparer: &F, state: &S) -> Option<T>
    where
        F: Fn(&T, &T, &S) -> bool,
    {
        // 1. pop target
        // 2. swap tail
        if let Some(removed_node) = self.swap_remove(index) {
            // 3. sink down
            let mut sink_index = index;
            loop {
                let lchild_index = (sink_index << 1) + 1;
                let rchild_index = (sink_index << 1) + 1;
                let sink_value = &self.data[sink_index];
                let lchild_value = &self.data[lchild_index];
                let rchild_value = &self.data[rchild_index];

                // Find highest node.
                let (should_sink, new_index) = if rchild_index < self.size {
                    // if have l,r child
                    match comparer(lchild_value, rchild_value, state) {
                        true => (comparer(lchild_value, sink_value, state), lchild_index),
                        false => (comparer(rchild_value, sink_value, state), rchild_index),
                    }
                } else if lchild_index < self.size {
                    // if l child only
                    // sink if lchild > value
                    (comparer(lchild_value, sink_value, state), lchild_index)
                } else {
                    // if no child
                    // no need to sink
                    (false, 0)
                };

                // Sink down.
                if should_sink {
                    self.data.swap(sink_index, new_index);
                    sink_index = new_index;
                    // update sink index that should sink next time.
                } else {
                    break;
                }
            }
            Some(removed_node)
        } else {
            // index out of range
            None
        }
    }

    pub fn peek(&self) -> Option<&T> {
        self.peek_at(0)
    }

    fn peek_at(&self, index: usize) -> Option<&T> {
        if index < self.size {
            Some(&self.data[index])
        } else {
            None
        }

    }

    pub fn as_slice(&self) -> &[T] {
        unsafe {
            std::slice::from_raw_parts(self.data.as_ptr(), self.size)
        }
    }

    /// add a elem to the tail of underlaying array.
    /// return Some(value) if full, where value is the input value.
    fn add_last(&mut self, value: T) -> Option<T> {
        if self.size == N {
            return Some(value);
        }

        self.data[self.size] = value;
        self.size += 1;
        None
    }

    /// swap up tail to index and remove the index elem
    /// return None if out of range
    fn swap_remove(&mut self, index: usize) -> Option<T> {
        if index < self.size {
            self.size -= 1;
            let removed = self.data[index];
            // swap up the tail, which should sink down out side.
            let tail = self.data[self.size];
            self.data[index] = tail;

            Some(removed)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use crate::FixedHeap;

    #[test]
    fn test_push_peek_pop() {
        let mut heap: FixedHeap<i32, 16> = FixedHeap::new();
        let comparer = |a: &i32, b: &i32, _: &()| a > b;
        assert_eq!(None, heap.peek());
        assert_eq!(heap.push(1, &comparer, &()), None);
        assert_eq!(Some(&1), heap.peek());
        assert_eq!(heap.push(3, &comparer, &()), None);
        assert_eq!(Some(&3), heap.peek());
        assert_eq!(heap.push(2, &comparer, &()), None);
        assert_eq!(Some(&3), heap.peek());
        /*
         *    3
         *  1   2
         */
        assert_eq!(Some(3), heap.pop(&comparer, &()));
        assert_eq!(Some(&2), heap.peek());
        assert_eq!(Some(2), heap.pop(&comparer, &()));
        assert_eq!(Some(&1), heap.peek());
        assert_eq!(Some(1), heap.pop(&comparer, &()));
        assert_eq!(None, heap.peek());
        assert_eq!(None, heap.pop(&comparer, &()));
    }

    #[test]
    fn test_push_full() {
        let mut heap: FixedHeap<i32, 4> = FixedHeap::new();
        let comparer = |a: &i32, b: &i32, _: &()| a > b;
        assert_eq!(heap.push(1, &comparer, &()), None);
        assert_eq!(heap.push(2, &comparer, &()), None);
        assert_eq!(heap.push(4, &comparer, &()), None);
        assert_eq!(heap.push(3, &comparer, &()), None);
        assert_eq!(heap.push(5, &comparer, &()), Some(1));

        assert_eq!(Some(5), heap.pop(&comparer, &()));
        assert_eq!(Some(4), heap.pop(&comparer, &()));
        assert_eq!(Some(3), heap.pop(&comparer, &()));
        assert_eq!(Some(2), heap.pop(&comparer, &()));
        assert_eq!(None, heap.pop(&comparer, &()));
    }

    #[test]
    fn test_push_pop_equal() {
        let mut heap: FixedHeap<i32, 4> = FixedHeap::new();
        let comparer = |a: &i32, b: &i32, _: &()| a > b;
        assert_eq!(heap.push(7, &comparer, &()), None);
        assert_eq!(heap.push(7, &comparer, &()), None);
        assert_eq!(heap.push(7, &comparer, &()), None);

        assert_eq!(Some(7), heap.pop(&comparer, &()));
        assert_eq!(Some(7), heap.pop(&comparer, &()));
        assert_eq!(Some(7), heap.pop(&comparer, &()));
        assert_eq!(None, heap.pop(&comparer, &()));
    }

    #[test]
    fn test_keys() {
        let mut heap: FixedHeap<usize, 4> = FixedHeap::new();
        fn comparer(a: &usize, b: &usize, state: &[i32; 4]) -> bool {
            state[*a] > state[*b]
        }
        let state = [1, 3, 1, 2];
        assert_eq!(heap.push(0, &comparer, &state), None);
        assert_eq!(heap.push(1, &comparer, &state), None);
        assert_eq!(heap.push(3, &comparer, &state), None);

        assert_eq!(Some(1), heap.pop(&comparer, &state));
        assert_eq!(Some(3), heap.pop(&comparer, &state));
        assert_eq!(Some(0), heap.pop(&comparer, &state));
        assert_eq!(None, heap.pop(&comparer, &state));
    }


    #[test]
    fn test_as_slice() {
        let mut heap: FixedHeap<i32, 16> = FixedHeap::new();
        let comparer = |a: &i32, b: &i32, _: &()| a > b;
        assert_eq!(heap.push(7, &comparer, &()), None);
        assert_eq!(heap.push(9, &comparer, &()), None);
        assert_eq!(heap.push(2, &comparer, &()), None);
        assert_eq!(heap.push(5, &comparer, &()), None);
        assert_eq!(heap.push(8, &comparer, &()), None);
        assert_eq!(heap.push(8, &comparer, &()), None);
        assert_eq!(heap.push(3, &comparer, &()), None);
        /*
         *
         *      9
         *   8     8
         * 5  7  2   3
         *
         */

        let slice = heap.as_slice();
        assert_eq!(7, slice.len());
        assert_eq!(9, slice[0]);
        assert_eq!(8, slice[1]);
        assert_eq!(8, slice[2]);
        assert_eq!(5, slice[3]);
        assert_eq!(7, slice[4]);
        assert_eq!(2, slice[5]);
        assert_eq!(3, slice[6]);
    }
}
