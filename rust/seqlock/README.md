# seqlock

learn from [seqlock](https://github.com/Amanieu/seqlock)

writer之间利用分布式系统中的一致性思想，**解决了rwlock中的reader和writer互斥的问题**, 只保留了writer之间的互斥。

**适用于writer较少的场景**

## usage

```rust
use seqlock::SeqLock;

let lock = SeqLock::new(5);

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
```

## impl

ref: https://zhuanlan.zhihu.com/p/94713372

linux中的seqlock。利用分布式系统中的一致性思想，**解决了rwlock中的reader和writer互斥的问题**, 只保留了writer之间的互斥。

1. write lock时, 上锁, ***seq++*
2. write unlock时, 解锁, ***seq++*
3. 初始时seq为偶数, 因此**在上锁时seq为奇数**
4. reader读取时判断seq是否为奇数, 如果为奇数则等待，直到seq变成偶数
5. reader读取后还有检查读取后的seq和读取前的seq是否一致, 如果不一致则说明发生了write, 需要重试
6. seq加减使用无锁的加减

## rust api


- `MaybeUninit<T>`
    * 内存布局和T相同
    * 使用MaybeUninit来封装**可能未初始化的变量**, 以便避免编译问题(如被编译器自动drop)
    * 因为rust使用的引用必须是内存对齐和**初始化的**, 对于从堆分配到的数据, 可能只是开辟了空间但没有初始化
- `ptr::read_volatile(self.data.get() as *mut MaybeUninit<T>)`
- `fence(Ordering::Acquire)`
    * https://rustwiki.org/zh-CN/std/sync/atomic/fn.fence.html
    * 防止编译器和CPU的重排(乱序发射)
    * Ordering::Relaxed: 随意重排序
    * Ordering::Acquire: "需要保证顺序", 细节学习TODO
- SeqLockGuard
    * 使用RAII的方式管理锁以实现自动上锁和释放
    * TODO RAII学习
- crate `parking_lot`
- `into_inner`
    * Unwraps the value, consuming the cell



