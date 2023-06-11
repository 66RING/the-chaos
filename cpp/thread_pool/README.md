# 线程池实现

1. N个worker线程不断处理任务队列中用户提交的任务
    - 等待条件变量通知任务到来或stop
2. 用户提交任务
    - 原子提交
    - 通知一个worker条件变量
3. 停止信号
    - 更新标志位并唤醒所有worker


## cpp misc

- lock
    * `unique_lock`和`lock_guard`的区别
        + `unique_lock`更灵活, 可以移动和手动lock/unlock; 而`lock_guard`RAII自动上锁和出作用域自动解锁, 不能手动解锁
    * `unique_lock`和`shared_lock`的区别
        + 互斥锁和读写锁的区别
- mutex
    * mutex和lock的区别: lock是对mutex的封装, 自动unlock等
- cv 条件变量
    * 外部传入lock保证了条件判断的原子性
    * 然后换内部锁保证加入等待队列的原子性
- 可变参数
    * 通过`...`传参, `f(int n, ...)`, 然后可以使用api获取参数, `va_list`等
- 可变参数**模板**, 三种地方使用`...`
    1. `typename ...Ts`, 表示多种type
    2. 形参处`Ts... args`, 表示多个参数
    3. 实参处`args...`, 表示参数列表
- 完美转发
    * 利用模板参数推导规则实现的小trick: `forward`
- bind: 函数适配器, 生成一个新的可调用对象, `bind( F&& f, Args&&... args )`
    * 将参数绑定到f上


### 条件变量的实现

- 本质: 
    * 外部传入lock保证了条件判断的原子性
    * 然后换内部锁保证加入等待队列的原子性

```cpp
template <typename _Lock, typename _Predicate>
void wait(_Lock &__lock, _Predicate __p) {
  while (!__p())
    wait(__lock);
}

template <typename _Lock> void wait(_Lock &__lock) {
  shared_ptr<mutex> __mutex = _M_mutex;
  unique_lock<mutex> __my_lock(*__mutex);
  _Unlock<_Lock> __unlock(__lock);
  // *__mutex must be unlocked before re-locking __lock so move
  // ownership of *__mutex lock to an object with shorter lifetime.
  unique_lock<mutex> __my_lock2(std::move(__my_lock));
  _M_cond.wait(__my_lock2);
}
```

即对于`wait(&lock)`传入的锁, 在阻塞让出前会释放, 恢复后再次加锁。

```cpp
template <typename _Lock, typename _Predicate>              // {{{ lock
void wait(_Lock &__lock, _Predicate __p) {
  while (!__p())  // lock保证了条件判断的原子性
    wait(__lock);
}

template <typename _Lock> void wait(_Lock &__lock) {
  shared_ptr<mutex> __mutex = _M_mutex;
  unique_lock<mutex> __my_lock(*__mutex);
  _Unlock<_Lock> __unlock(__lock);                         // }}} unlock
  // *__mutex must be unlocked before re-locking __lock so move
  // ownership of *__mutex lock to an object with shorter lifetime.
  unique_lock<mutex> __my_lock2(std::move(__my_lock));
  _M_cond.wait(__my_lock2);
}
```


## Q

wait会拿到scope lock然后让出thread, 那wait是怎么不让这个scope lock影响到下面的代码的?

`unique_lock`可以手动加锁解锁, 保证加入任务队列完成后，在切换任务前会释放锁，恢复后再重新上锁。

