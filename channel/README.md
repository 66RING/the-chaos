# channel实现

- Usage: `let (tx, rx) = flume::unbounded();`

总结

- 本质就是一个queue, 然后有send/recv的接口
- 基本构成:
    * cap, 通道的最大容量
    * queue, 已经send出, 等待消耗(recv)的msg
        + queue一直都是"msg buffer", 所以是用来存储send出的数据的
    * sending, 正在发送的sender: 阻塞直到recv
    * waiting, 正在接收的recver: 阻塞直到send
- 基本机制: 
    * send/recv的阻塞都是在等待另一端的处理
    * recv空时要等待send, 通知自己的Hook插入到waiting中。send在处理时会获取waiting中的Hook, send出后通过Hook发送通知, recv端恢复
    * recv满时要等待send, 通知自己的Hook插入到sending中。recv在处理时会获取sending中的Hook, recv出后通过Hook发送通知, send端恢复
- 关键问题/设计
    * 怎么mpmc?
        + Arc加锁
    * 怎么做async?
        + 实现Future trait和Sink
        + **TODO** the rust magic
    * 做了什么优化?
        + TODO: 需要对比其他设计
- Project trcik
    * 过程通过参数传入, e.g.`make_signal()`, `do_block()`, 更加灵活。同理rcore中文件系统read/write一个block的一个设计


## 实现

### signal.rs

> **类似单一资源信号量**

对`std::Thread`封装, 抽象出一个trait, 本质就是让出thread

```rust
SyncSignal(Thread);
```

- `fire()` -> 相当于V操作, 释放资源
- `wait()` -> 相当于P操作, 等待资源

> https://rustwiki.org/zh-CN/std/thread/fn.park.html

- `Thread::park()`阻塞当前线程
- `Thread::unpark()`通过句柄恢复一个线程


### select.rs

```rust
struct SelectSignal (
    thread::Thread, // thread handle
    Token,  // usize
    AtomicBool,
    Arc<Spinlock<VecDeque<Token>>>,
)
impl Signal for SelectSignal {}


trait Selection {
    fn init(&mut self) -> Option<T>;
    fn poll(&mut self) -> Option<T>;
    fn deinit(&mut self);
}

/// 通过一个Selector等待多个阻塞事件
/// flume::Selector::new()
///     .recv(&rx0, |b| println!("Received {:?}", b))
///     .recv(&rx1, |n| println!("Received {:?}", n))
///     .wait();

struct Selector {
    selections: Vec<Box<dyn Selection<'a, T> + 'a>>,
    next_poll: usize,
    signalled: Arc<Spinlock<VecDeque<Token>>>,
    #[cfg(feature = "eventual-fairness")]
    rng: nanorand::WyRand,
    phantom: PhantomData<*const ()>,
}
```

#### Selection

TODO:

- init
    * 首次执行send/recv
    * TODO: send/recv的实现, 阻塞? 失败跳过?
- poll
    * 再次尝试send/recv
- deinit
    * TODO


#### Selector

> 一次等待多个阻塞事件的抽象
>
> A type used to wait upon multiple blocking operations at once.

- `send`
    * 记录一个Selection到vector中
    * `SendSelection`
        + `poll`: check hook and call mapper. TODO
- `recv`
    * 记录一个Selection到vector中
- `poll`
    * 挨个对Selection执行poll尝试send/recv
- `wait` -> `wait_inner`
    1. 随机起点一个`next_poll`尝试init(第一次send/recv), 后面循环遍历vector(`next+1 % len`)
    2. 执行poll, 挨个再次尝试send/recv
    3. loop, 使用信号(park/unpark)阻塞/唤醒
    4. Selection挨个deinit


### async.rs

- struct AsyncSignal
    * fire: 把woken置true, 然后wake task

- Sender(async部分)
    * `send_async`: 包裹到`SendFut`等待调度
    * `sink`: 创建`SendSink`
- SendFut: impl Future for SendFut
    * poll: 
        + queue中有两种状态: QueuedItem和NotYetSent
            + NotYetSent就第一次send, 然后设置各种回调(mapper, hook)
            + QueuedItem就唤醒
    * TODO: `reset_hook`
- SendSink: impl Sink for SendSink
    * TODO: pin.poll
- Receiver -> RecvFut -> impl Future
    * poll -> inner_poll
        + 有hook: TODO: review after hook
        + 没hook: 第一次做recv, 设置回调函数(mapper, hook等)
- RecvStream: impl Stream
    * `poll_next`: 同poll, 只是AsyncSignal::new时`is_stream`bool置true


#### Sink

可以异步的方式发送到Sink中, 类似写buffer和落盘

- 初始化send
- poll直到完成

TODO


### lib.rs

#### Hook

```rust
struct Hook<T, S: ?Sized>(Option<Spinlock<Option<T>>>, S);
```

- **`struct Hook<T, S: ?Sized>(Option<Spinlock<Option<T>>>, S)`**: (msg, signal)
    * slot(): new一个msg + signal
    * trigger(): new一个仅signal
    * `fire_recv`获取Hook中的msg和signal
    * `fire_send`将msg存入Hook中, 并返回存入的结果msg和signal
    * `wait_recv`尝试获取Hook中的msg, 否则等待signal
    * `wait_send`等待Hook中的msg被取出(置None), 否则等待signal

#### Chan

```rust
struct Chan<T> {
    /// (channel的最大容量, Hook queue)
    /// 正在阻塞的send端
    sending: Option<(usize, SignalVec<T>)>,
    /// 待消耗(recv)的msg queue
    queue: VecDeque<T>,
    /// 正在阻塞的recv端
    waiting: SignalVec<T>,
}
```

- `pull_pending`: 遍历所有sending, 接收, 存起来到queue中, 然后发送信号通知


#### Shared

```rust
struct Shared<T> {
    chan: ChanLock<Chan<T>>,
    disconnected: AtomicBool,
    sender_count: AtomicUsize,
    receiver_count: AtomicUsize,
}
```

- `send()`
    * TODO: 没看懂, 都有把msg存入queue


```rust
fn send<S: Signal, R: From<Result<(), TrySendTimeoutError<T>>>>(
    &self,
    msg: T,
    should_block: bool,
    make_signal: impl FnOnce(T) -> Arc<Hook<T, S>>,
    do_block: impl FnOnce(Arc<Hook<T, S>>) -> R,
) -> R;
```

- `recv`
    1. `pull_pending`尽可能接收消息, 保存到queue中
    2. 尝试从queue中取出消息并返回
    3. 否则没有信息尝试`do_block`: 将`make_signal` hook加入waiting中, 然后`do_block(hook)`
        - `make_signal -> Hook`, 相当于Hook的构造方法
        - `do_block(Hook)` 相当于Hook的使用方法

```rust
fn recv<S: Signal, R: From<Result<T, TryRecvTimeoutError>>>(
    &self,
    should_block: bool,
    make_signal: impl FnOnce() -> Arc<Hook<T, S>>,
    do_block: impl FnOnce(Arc<Hook<T, S>>) -> R,
) -> R;
```

> /// Send a value into the channel, returning an error if all receivers have been dropped.
> /// If the channel is bounded and is full, this method will block until space is available
> /// or all receivers have been dropped. If the channel is unbounded, this method will not
> /// block.

- `send_sync`主要要考虑drop, full的情况
    * 满了要block, drop了要unblock, unbound不会block



- `recv_sync`

#### Sender

- `self.shared.send_sync`

#### Receiver

- `self.shared.recv_sync`


#### misc

- `wait_lock`: 递进退让
    * spin版: 有`try_lock`
        + 拿不到锁就`yield`
        + 连续10次就sleep, 没轮迭代sleep的时间都会增加, 最多sleep 1ms
    * 非spin版
        + 直接`lock.lock()`





# Trick

> TODO magic

- `PhantomData<T>`
- [`Waker`](https://doc.rust-lang.org/nightly/core/task/wake/struct.Waker.html)
- [Sink](https://docs.rs/futures-sink/latest/futures_sink/trait.Sink.html)






