# llm-sdk

> [ref](https://github.com/tyrchen/llm-sdk)

- trick
    * anyhow
    * tracing + trace subscriber
    * clippy
    * `derive_builder`
        + `#[builder(deault, setter(into))]`
        + default自动调用default trait, 或者自定义default = xxx
    * `#[serde(rename_all="snake_case")]`, `#[serde(rename = "XXX")]`
    * 设计模式
        + builder pattern: 可以使用derive builder快速实现

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


## tool

> 定义一些function, 让GPT决定调用哪个。MemGPT

- Q: 为什么需要schema
- A: 因为OpenAI的API接受的是一个json schema, 而不是json字符串
- Q: 为什么不serde直接反序列化
- A: 因为serde反序列化得到是json字符串和json schema是两回事, schema更像是对对象的描述https://json-schema.org/understanding-json-schema

- 如何处理复杂的json
    * 一方面如何处理复杂json: schemars
    * 另一方面怎么让用户方便使用

多trait写法

```rust
pub trait ToSchema: JsonSchema {
    fn to_schema() -> serde_json::Value;
}
```

## Speech

> 语音


## transcriptions

> 语言转文字

**处理表单的情况**, 序列化反序列化

- reqwest.multipart
- strum

### crate

- Strum: 把enum转string
    * 为什么要用strum: 因为serde对单个字段的转换不方便也不方便直接转换成表单。所以就用strum一个一个填写表单。


## Embeddings

### crate

- serde
    * untagged: 不让再添加一层


## retry

添加重试中间件

这种web框架的中间件都相当于一个gate, 用于决定走这个中间件还是直接next。所以handle接口方法就是有个next用于决定是否往下走。

```rust
impl Middleware for RetryMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response>;
}
```

具体这里的可以`Request`, `Extensions`, `Next`都是什么, 可以rust doc里查看source, 看一个源码use的哪些crate, cv一下

### crate

- `reqwest-retry`
- `reqwest-middleware`
- `reqwest-tracing`
- impl trait技巧: rust doc里查看source, 看一个源码use的哪些crate, cv一下







