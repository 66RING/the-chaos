use core::mem::align_of;

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
