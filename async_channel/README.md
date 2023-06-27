# Async-channel

TODO: Async体现在哪


使用: `send`, `recv`但async

- 事件驱动: `event_listener` crate
- Sender
- Receiver

- 无锁编程做ref count


```rust
let (s, r) = async_channel::unbounded();

assert_eq!(s.send("Hello").await, Ok(()));
assert_eq!(r.recv().await, Ok("Hello"));
```

```rust
struct Channel<T> {
    /// Inner message queue.
    queue: ConcurrentQueue<T>,

    /// Send operations waiting while the channel is full.
    send_ops: Event,

    /// Receive operations waiting while the channel is empty and not closed.
    recv_ops: Event,

    /// Stream operations while the channel is empty and not closed.
    stream_ops: Event,

    /// The number of currently active `Sender`s.
    sender_count: AtomicUsize,

    /// The number of currently active `Receivers`s.
    receiver_count: AtomicUsize,
}
```


```rust
// Future object
pub struct Send<'a, T> {
    sender: &'a Sender<T>,
    listener: Option<EventListener>,
    msg: Option<T>,
}
```

- Sender
    * `try_send`: 入队, 通知事件
    * `send`: 返回一个Send对象, 同时是个**Future对象**, 调用await才真正执行
    * `send_blocking`: 生成send对象并触发await执行
    * 满不阻, 返err

```rust
pub struct Sender<T> {
    /// Inner channel state.
    channel: Arc<Channel<T>>,
}
```

- Receiver
    * `try_recv`: 出队, 通知事件
    * `recv`: 返回Recv的Future对象
    * `recv_blocking`: 生成recv对象并触发await执行
    * 空阻塞, 等send

```rust
pub struct Receiver<T> {
    /// Inner channel state.
    channel: Arc<Channel<T>>,

    /// Listens for a send or close event to unblock this stream.
    listener: Option<EventListener>,
}
```


## Stream

`impl<T> Stream for Receiver<T>`

- `poll_next`

loop有消息时接收并返回, 没消息时走到等待的分支

```rust
loop {
    // 有等待事件则等待
    if listener() {
        wait();
        clear_listener();
    }

    loop {
        if msg = recv() {
            clear_listener();
            return msg
        }
        // 接收失败则创建等待
        create_listener() && break;
    }
}
```



## Future

### Send / Recv

```rust
pub struct Send<'a, T> {
    sender: &'a Sender<T>,
    // listener为空表示无需等待
    listener: Option<EventListener>,
    msg: Option<T>,
}

pub struct Recv<'a, T> {
    receiver: &'a Receiver<T>,
    listener: Option<EventListener>,
}
```

- `run_with_strategy`
    * 尝试recv, 成功就将msg返回
    * 否则进入等待分支
        + 创建listener
        + poll listener: 完成返Ok, 还有返Err(l) listener
- `wait`
    * -> `run_with_strategy`

### impl Future

- `poll`接口 -> `run_with_strategy` -> `Strategy::poll`


### impl Strategy

> 实现具体的poll方法

- NonBlocking
    * 使用EventListener的poll
- Blocking
    * 使用EventListener的wait


## Misc

### Pin

- 用法与只能指针相同
- 保证对象不会move
    * 不让用户获取底层指针, 如使用`mem::swap`等内存操作

#### 为什么Pin在Async中如此重要

> 主要用于解决引用自身的问题。如, 当这种结构发生拷贝时, 引用指向就存在内存安全问题, 如A1拷贝到A2, 而A2的`ref_a`仍指向A1.a

```rust
struct A1 {
    a,
    ref_a,
}

struct A2 {
    a,
    ref_a,
}
```

编译器基于async和await自动生成异步代码, 创建状态机:

- 保存状态和变量(因为编译器知道会用到什么变量): `struct XXX_State {}`

```rust
match self.state {
    State1 => {
        match poll() {
            Ready() => {}
            Pending() => return Pending,
        }
        self.state.update();
    },
    State2 => {
        match poll() {
            Ready() => {}
            Pending() => return Pending,
        }
        self.state.update();
    },
}
```

因为rust自动根据上下文生成结构体, 而上下文中可能出现对局部变量的引用, 所以最后生成的结构体中就可能出现自引用。 



