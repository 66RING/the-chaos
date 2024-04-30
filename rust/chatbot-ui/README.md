# chatbot-ui template

- [freya](https://github.com/marc2332/freya)
- [dioxus](https://dioxuslabs.com/learn/0.5/getting_started/)


## dev log

[getting started](https://book.freyaui.dev/getting_started.html)

1. 使用`state = use_signal(|| 0)`创建状态, e.g. 计数器初始化为0
2. 使用`state()`, `state.read()`订阅/读取状态
3. 使用`*state.write()`修改状态
4. 传入`rsx!()`进行渲染

```rust
use freya::prelude::*;

fn app() -> Element {
    // 用use_signal创建共享状态的state
    let mut state = use_signal(|| 0);

    let onclick = move |_| {
        // signal提供了一些方法来更新状态
        state += 1;
        // 也可以用这种方式
        *state.write() += 1;
    };
    println!("{}", state());

    rsx!(
        label {
            onclick,
            "State is {state}"
        }
    )
}


fn main() {
    launch(app);
}
```

