# Ava-bot

1. app构思: 长什么样/表现形式画一画
2. 功能需求: 需要用哪些功能
    - 浏览器录音...


- 数据结构需求分析
    * ⭐Dashmap, 相当于`Mutex<Arc<HashMap>>`


- 技术
    * ⭐SSE(server side event)
        + api调用链那么长, 需要一些机制防止出错
    * mpsc
    * htmx sse
    * sse
    
## 1. basic

### crate

- clap
- rustls
- askama
- sse

### tools

- mkcert, 制作本地https证书
- watchexec, 热更新


