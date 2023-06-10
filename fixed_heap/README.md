# fixed heap

> 固定大小。backed by array。用户可自定义比较函数, 甚至可以根据value对key排序。使用unsafe代码提升性能。
>
> 0 base 抽象"大"顶堆(大是用户定义compare的大), 抽象大顶堆, 非二叉搜索堆, 只要求root大于两孩子
>
> 固定大小, 满插入则淘汰"最小"

```rust
// index be like:
//        0
//     1     2
//   3   4 5   6
```

- Usage
    - `let mut heap: FixedHeap<usize, 16> = FixedHeap::new();`
    - `pub fn push<S, F>(&mut self, value: T, comparer: &F, state: &S) -> Option<T>`
    - `pub fn pop<S, F>(&mut self, comparer: &F, state: &S) -> Option<T>`

数据结构就是一个固定大小的数组 + 一些元数据

```rust
// 直接N静态值作为最大容量依据, 甚至不需要成员变量记录最大容量
pub struct FixedHeap<T, const N: usize> {
    // 当前数据量
    high: usize,
    data: [MaybeUninit<T>; N],
}
```

> You can think of `MaybeUninit<T>` as being a bit like `Option<T>` but without
> any of the run-time tracking and without any of the safety checks.

可以根据value排序, 不单单是key的原理: 

`fn comparer(a: &usize, b: &usize, state: &[i32; 4]) -> bool`, compare可以额外传入一个全局state, 自定义排序是用index到的key索引state排序

## push

> **大顶堆的插入算法**

- 大顶堆的插入: 尾插, 上浮
- fixed-heap下需要考虑淘汰: 从最底层淘汰一个
    * 找最尾元素的父节点开始找最小节点: `for i in (N >> 1) .. N`
        + N >> 1相当于N / 2向下取整
        + 并且最小节点一定在`(N >> 1) .. N`中, 因为如果N >> 1 父节点在左子树, index++会包含右子树; 如果N>>1在右子树, 则左子树一定是满二叉树, 即可以跳过左子树的父节点
    * 如果待插入元素更小则插入失败

如果空间充足返回None, 否则返回淘汰的节点Some(value)


## pop

> 大顶堆的pop: pop root, 最小换到root, sink

- pop root, swap tail, sink down
- `pop_at(0)`


## 迭代器

直接顺序返回底层array元素


## misc

- `as_slice`, 直接返回底层array slice
    * 返回`&[T]`
- `copy_from_slice`, slice直接拷贝到底层slice, 不做任何位置调整


## rust tips

- 使用`mem::replace()`做内存替换
- `as_slice`用法, 使用`std::slice::from_raw_parts()`传入一个ptr
- 为何使用`MaybeUninit`? [TODO](https://learnku.com/articles/65520)
    1. "RUST 的引用使用的内存块必须保证是内存对齐及赋以初始值，未初始化的内存块和清零的内存块都不能满足引用的条件。但堆内存申请后都是未初始化的，且在程序中某些情况下也需要先将内存设置为未初始化，尤其在处理泛型时。因此，RUST 提供了 MaybeUninit 容器来实现对未初始化变量的封装，以便在不引发编译错误完成对 T 类型未初始化变量的相关操作."
        - 对未初始化的内存进行保护, 节省初始化的时间?
        - 对申请的内存都是未初始化的, 可以节省初始化的时间
    2. rust编译器不再自动管理drop



