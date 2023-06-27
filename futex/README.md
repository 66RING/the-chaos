# Futex Lock

- `wait(atomic: &AtomicU32, value: u32)`, 等待直到两者**不相等**
- `wake_one(atomic: *const AtomicU32)`, 唤醒一个正在等待这个Atomic的thread
- `wake_all(atomic: *const AtomicU32)`, 唤醒所有正在等待这个Atomic的thread
- wake的作用是唤醒wait(信号量PV操作)

本质上是使用[futex](https://man7.org/linux/man-pages/man2/futex.2.html)系统调用完成用户态上锁和通知, 是一种用户态和内核态混合的同步机制。

而futex的本质是单资源的信号量, 没资源时P操作会进入等待, V释放时有等待则唤醒。其优化点在于使用原子操作(无锁操作)和共享内存减少了内核态的陷入。传统的进程间通信需要一定的共享状态才能看到彼此，这往往是要经过内核的，如信号量操作要陷入内核态。而在竞争少的情况下也要频繁陷入内核去做检测开销是很大的。所以futex的做法是：

- 使用mmap开辟共享内存, 让状态信息直接用户态可读
- 使用原子操作修改条件变量
- 出现竞争需要等待时才陷入内核态，让内核管理等待队列的入队和唤醒

```c
long syscall(SYS_futex, uint32_t *uaddr, int futex_op, uint32_t val,
             const struct timespec *timeout,   /* or: uint32_t val2 */
             uint32_t *uaddr2, uint32_t val3);
```

主要需要关注`futex_op`参数, Futex operations章节相关的描述。

- `FUTEX_WAIT`: 原子性的检查uaddr中计数器的值是否为val,如果是则让进程休眠，直到`FUTEX_WAKE`或者超时(time-out)。也就是把进程挂到uaddr相对应的等待队列上去
- `FUTEX_WAKE`: 最多唤醒val个等待在uaddr上进程










