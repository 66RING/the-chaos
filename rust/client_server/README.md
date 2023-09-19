# client server框架模板

> 普通版 + async版

目录结构

```
src
├── bin
│   ├── cmd-client.rs
│   └── cmd-server.rs
├── client.rs
└── server.rs
```

- `bin/cmd-client.rs`和`bin/cmd-server.rs`直接和用户打交道
- `client.rs`和`server.rs`相当于库函数, 用面向对象的思想给CMD提供API

## client

1. 解析命令行参数
    - TODO: {clap版, clap + structopt版} + {RPC, TCP}
    - 常用的有server的地址:端口, 发送的请求
2. 根据命令行参数执行对应的操作
    - connect到server, 获取连接对象(client)
    - 使用连接对象的API发送请求, 如client.get()


## server

1. 创建tcp连接
2. listen监听到来的连接, 作为connection/stream传递后续处理(serve)
3. 根据connection获取地址信息, 读写对象
4. while循环读取请求信息
5. 根据请求信息的元数据做相应的处理


## overview

cmd-client.rs

```rust
/// 命令行参数
/// enum或者struct
enum Command {}

fn main() { run() }
fn run() {}
```

client.rs

```rust
/// 连接对象, 提供API
pub struct Client {
    // 消息都是根据协议序列化后发送的, reader需要反序列化
    reader: Deserializer<IoRead<BufReader<TcpStream>>>,
    // 将消息序列化后写入writer
    writer: BufWriter<TcpStream>,
}

impl Client {
    // 对象构建方法
    pub fn connect() -> Client {
    }

    // 方便的API访问
    pub fn get() -> Result<()> {
    }
}
```

cmd-server.rs

```rust
/// server启动需要的命令行参数
struct Opt {}

fn main() { run() }
fn run() {}
```

server.rs

```rust
/// server对象
pub struct Server {}

impl Server {
    /// 创建server对象
    pub fn new() {}
    /// 启动server东西
    pub fn run() {}
    /// 处理一个connection
    pub fn serve() {}
}
```

common.rs, 通信协议, 请求/回复等的编码

## clap

## structopt

## tokio

## RPC

