# server

1. 构建单例process param对象
    - `the_process_param()`
    - 参数解析
        * TODO
2. 使用process param对象初始化
    - `init.cpp::init()`
        * 加载位置文件: `ini.h::class Ini`
        * **初始化全局对象: Buffer管理器, Handler等**
            + Handler负责处理真正的操作, 如`drop table`等. `DefaultHandler()`
        * 构建seda
            + TODO
            + 工程模式: why?
3. init server: `init_server`
    - 初始化server参数: `ServerParam`. addr, port, 最大连接数等 
    - 构建server: `new Server(param)`
4. serve: `Server::serve()`
    - `start()`启动监听事件并在后续使用事情驱动
        * `Server::start_tcp_server()`
            + create socket
            + set reuse addr
            + set non block: `fcntl(fd, F_GETFL)`
            + bind
            + listen
            + add listen event
        * (类似redis), 不想用event库则用原生epoll代替
        * accept()绑定到listen的fd
            + 构建connection对象, 记录fd足以, 但可以记录其他信息, e.g. event, addr, mutex...
            + 给客户端对象创建监听事件, 并添加到主循环中。因为是recv所以是read事件, 绑定到`Server::recv`处理
        * `recv()`绑定到接收到客户请求事件, **仅做数据接收**
            + new buffer and read from fd(**注意分段read**)
            + 数据保存到client buffer传入SessionEvent做进一步处理: **添加到stage调度系统**
        * `SessionStage::handle_event`
            + `recv()`收到请求数据后通过`session_stage_->add_event(sev)`通知session stage处理请求数据
            + 之后会调用`SessionStage::handle_request`完成请求的处理
                + get client buffer
                + 调用执行器(也是通过stage事件驱动)
    - `event_base_dispatch()`执行主循环(event库)
5. stage调度系统:
    a. 注怎么找到对应handler的? 虚函数, 各自实现`handle_event`接口
        i. 入口: `dep/common/thread_pool::run_thread`
        * `Stage`类有个`handle_event`虚函数, 各个event handler实现它, 最后会只用父类统一接口调用到
            + `virtual void handle_event(StageEvent *event) = 0`
    b. 什么时候把Stage子类传进去的? `SessionStage::handle_event`
        - server有一个全局的静态对象: `Server::session_stage_`用于接收用户请求
        - 在`init.cpp::prepare_init_seda()`阶段会创建SessionStage工厂函数
        - 在`Server::init()`阶段会通过字符串获取到`SessionStage`对象保存到全局`Server::session_stage_`中
6. 关闭
    - 当read()返回的长度是0时说明连接关闭, `event_del`将event移除并关闭fd

- misc
    * **seda**!!

## libevent

> [Linux manual page](https://man7.org/linux/man-pages/man3/event.3.html)

- `event_base_new`
- `event_base_dispatch`
- `event_del`
- `event_free`
- `event_base_free`
- `event_base_loopexit`
    * 跟base event添加一个timeout属性
- `struct event *event_new(struct event_base *, int fd, short flags, event_callback_fn, void *)`
    * 在一个`event_base`集合上创建event, 绑定fd
    * 给事件绑定回调函数, 回调函数有固定的函数签名: `void f(int fd, short ev, void *arg);`
- `event_set`
    * 修改一个event的参数, 如绑定的fd, flags等
- `int event_base_set(struct event_base *, struct event *)`
    * 将一个event添加到一个`event_base`中
- `event_add`
    * 将event添加到main loop监听列表中

main loop基本结构:

1. 创建/接收一个event base
2. 根据需要监听的fd创建一个event: `event_new`
3. `event_add`添加新建的event
4. 执行循环`event_base_dispatch`


