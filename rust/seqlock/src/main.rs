use seqlock::Seqlock;

fn main() {
    let lock = Seqlock::new(5);

    {
        // Writing to the data involves a lock
        let mut w = lock.lock_write();
        *w += 1;
        assert_eq!(*w, 6);
    }

    {
        // Reading the data is a very fast operation
        let r = lock.read();
        assert_eq!(r, 6);
    }
}
