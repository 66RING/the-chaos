# offset of

C-Like `offset_of` functionality for Rust structs.

C中的实现是构造一个结构体(如在地址0处), 然后获取成员指针, 地址相减。如果是0地址处构建的结构体则成员地址就是offset。但在rust中, 有较多安全限制需要跨过。

- `offset_of!(struct_name, member)`, member到结构体开头的offset in byte
- `span_of!(struct_name, member1 .. member2)`, 两个member的跨度

可以考虑如下API, 不过要注意成员顺序可能会被编译器优化掉

```rust
#[derive(Default)]
struct A {
    u_8: u8,
}

fn main() {
    let a = A::default();
    println!("{}", align_of::<A>());
    println!("{:#?}", core::ptr::addr_of!(a.u_8));
    println!("{:#?}", core::ptr::addr_of!(a));
}
```

