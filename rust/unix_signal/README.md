# Signal Event

基本用法, 键盘事件触发后调用一个用户指定的回调函数。


```rust
pub fn set_handler<F>(mut user_handler: F) -> Result<(), Error>
where
    F: FnMut()
```

- 注意事项
    * 不同操作系统/API对signal的handler的继承情况不同

一个基本的原则是信号处理函数应该快速返回否则影响响应性，所以一个我们需要一个单独的thread来做信号处理。这样就需要一种机制来通知这个thread的执行。所以基本框架可以是:

1. 将信号的handler置为处理信号的发送, 如向一个pipe中非阻塞写数据, 从而恢复pipe读端(handler thread)的执行
2. 启动一个单独的thread做信号处理以保证响应性

可以考虑使用`nix::sys::signal::Signal`这个crate

## misc

- 善用文件描述符抽象相关API
    * fcntl
- pipe是buffered的
    * 写多少存多少, 读多少消耗多少, 可以以u8为单位一次一次触发读写


## TODO: Futex改造的可行性





