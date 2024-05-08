use parking_lot::{Mutex, MutexGuard};
use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ptr;
use std::sync::atomic::fence;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::thread;

pub struct Seqlock<T> {
    // sequence counter
    seq: AtomicUsize,
    // data store
    data: UnsafeCell<T>,
    // mutex guard
    mutex: Mutex<()>,
}

pub struct SeqlockGuard<'a, T> {
    _gurad: MutexGuard<'a, ()>,
    seqlock: &'a Seqlock<T>,
    // to save a atomic read
    seq: usize,
}

impl<T: Copy> Seqlock<T> {
    pub const fn new(val: T) -> Self {
        Seqlock {
            seq: AtomicUsize::new(0),
            data: UnsafeCell::new(val),
            mutex: Mutex::new(()),
        }
    }

    pub fn read(&self) -> T {
        loop {
            // read initial sequence number to check writer
            let seq1 = self.seq.load(Ordering::Acquire);

            // if seq1 is odd then it means writer currently modifying the value
            if seq1 & 1 != 0 {
                // yield cpu and retry
                thread::yield_now();
                continue;
            }

            // 1. Use volatile to always read from memory but not register, because writer may
            //    currently modifying.
            // 2. Use MaybeUninit to wrap a variable that may uninitialize and avoid compiler
            //    problem.
            let result = unsafe { ptr::read_volatile(self.data.get() as *mut MaybeUninit<T>) };

            // make sure seq2 read after reading data.
            // fence to avoid CPU/compiler reorder.
            fence(Ordering::Acquire);

            // if seq1 and seq2 is not the same then writer modified. retry
            let seq2 = self.seq.load(Ordering::Relaxed);
            if seq1 == seq2 {
                // extract from MaybeUninit
                return unsafe { result.assume_init() };
            }
            // retry
        }
    }

    fn begin_write(&self) -> usize {
        // seq++
        let seq = self.seq.load(Ordering::Relaxed).wrapping_add(1);
        self.seq.store(seq, Ordering::Relaxed);

        // make sure write data after seq++
        fence(Ordering::Release);

        seq
    }

    fn lock_guard<'a>(&'a self, guard: MutexGuard<'a, ()>) -> SeqlockGuard<'a, T> {
        // seq++
        let seq = self.begin_write();
        SeqlockGuard {
            _gurad: guard,
            seqlock: self,
            seq: seq,
        }
    }

    pub fn lock_write(&self) -> SeqlockGuard<T> {
        self.lock_guard(self.mutex.lock())
    }

    // Consumes this `SeqLock`, returning the underlying data.
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }

    pub fn get_mut(&self) -> &mut T {
        unsafe { &mut *self.data.get() }
    }
}

impl<T> Seqlock<T> {
    fn end_write(&self, seq: usize) {
        // The release ordering ensures that all writes to the data are done before writing the sequence number.
        self.seq.store(seq.wrapping_add(1), Ordering::Release);
    }
}

// impl auto deref
impl<'a, T: Copy + 'a> Deref for SeqlockGuard<'a, T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        unsafe { &*self.seqlock.data.get() }
    }
}

impl<'a, T: Copy + 'a> DerefMut for SeqlockGuard<'a, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.seqlock.data.get() }
    }
}

// impl auto drop
impl<T> Drop for SeqlockGuard<'_, T> {
    fn drop(&mut self) {
        self.seqlock.end_write(self.seq);
    }
}





