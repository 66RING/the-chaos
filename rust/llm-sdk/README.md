# llm-sdk

> [ref](https://github.com/tyrchen/llm-sdk)

- trick
    * anyhow
    * tracing + trace subscriber
    * clippy
    * `derive_builder`
        + `#[builder(deault, setter(into))]`
    * `#[serde(rename_all="snake_case")]`, `#[serde(rename = "XXX")]`
    * 设计模式
        + builder pattern

## create image

### project

* clippy: `cargo clippy`, 更加rust
- TODO: 学习一下
    * pre-commit-config

### 抽象

- 通过结构体生成request(builder模式)
    * 使用into抽象
        + 就是把结构体自身转换成json, 做serialize。然后添加到req的client中
- 定义好的东西, 如果有限可穷举, 我们最好都是用enum定义
- 设计模式
    * builder模式, 一个结构体的成员很多, 一个一个填充很麻烦的时候

### crate

- `derive_builder`
    - 一个结构内有很多参数, 自己构建new函数再一个一个赋值太麻烦了。可以使用`derive_builder`自动构建, 我们只需要描述好结构体就可以了。
- `serde`
    * 各种serde技巧, 自动serde json的, 跳过的, 默认的, rename的
    * 自动snake case等
- `anyhow`, 快速定义错误, 使用`anyhow!()`宏
    `return Err(anyhow::anyhow!("chat_completion error: {}", text));`

## chat completion

### crate

- serde enum tag
    * enum名自动添加成一个字段
- trace subscriber
    * **日志收集器**, 发明这个抽象的原因是因为日志可能出现在多个微服务中，需要解决分布式地日志处理
    * 需要添加RUST_LOG环境: `RUST_LOG=info`
    * 使用`info!()`, `error!()`等宏
    * 使用`#[ctor::ctor]`在程序执行器初始化`tracing_subscriber`
        + 日志收集器初始化后，才能用配套的日志记录API tracing记录


