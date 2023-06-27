# dufs

## Usage

> API部分比较重要, 其他大同小异

- server当前目录`dufs`
- server指定目录`dufs <dir>`
- API
    * 上传`curl -T path-to-file http://127.0.0.1:5000/new-path/path-to-file`
    * 下载`curl http://127.0.0.1:5000/path-to-file`
    * 删除`curl -X DELETE http://127.0.0.1:5000/path-to-file-or-folder`
    * 移动`curl -X MOVE https://127.0.0.1:5000/path -H "Destination: https://127.0.0.1:5000/new-path"`
    * 创建目录`curl -X MKCOL https://127.0.0.1:5000/path-to-folder`
    * 下载目录成zip`curl -o path-to-folder.zip http://127.0.0.1:5000/path-to-folder?zip`
    * 查找/列出内容
        ```
        curl http://127.0.0.1:5000?simple                 # output names only, just like `ls -1`
        curl http://127.0.0.1:5000?json                   # output paths in json format
        curl http://127.0.0.1:5000?q=Dockerfile&simple    # search for files, just like `find -name Dockerfile`
        ```
    * 权限认证
        ```
        curl --user user:pass --digest http://192.168.8.10:5000/file  # digest auth
        curl --user user:pass http://192.168.8.10:5000/file           # basic auth
        ```

## impl

- abs
    * 主循环`while {}`在哪? 
         + 就在在hyper里, 通过`make_service_fn`自动创建主循环
    * 如果只是call的话怎么同时处理静态页面和download等API?
        + `if method == Method::GET && self.handle_assets(req_path, headers, &mut res).await?`, 页面请求则走另一个分支, 但总体页面请求和API请求都是`call(req, remote_addr)`函数处理

- main.rs: 解析参数, 启动serve, tokio等待serve结束或结束信号(tokio简单`ctrl_c`)
    * `serve`: 给指定地址提供服务。使用hyper包创建MakeService对象`make_service_fn`, 然后使用tokio::spawn创建worker线程, 每个worker都是一个`hyper::Server::builder(incoming).serve(new_service)`
- args.rs: 负责参数定义和解析, 主要使用clap包
    * `build_cli`返回Command对象
    * `Args::parse(matches)`解析各个参数的值, 返回一个保存具体数据的Arg对象
- server.rs: 入口`hyper::Server::builder(accepter).serve(new_service)` -> `serve_func()` -> `inner.call(req, remote_addr)`
    * `Server::init(args.clone(), running)`创建Server对象, 主要提供一个`call(req, remote_addr)`接口用于处理外部请求, 使用外部传入的`running`作为停止标志位
    * `call(req, remote_addr)` -> `handle(req)`
        + 处理浏览器对页面数据的请求`handle_assets`: `if method == Method::GET && self.handle_assets(req_path, headers, &mut res).await?`
        + 指定路径是一个**文件**的情况直接下载文件: `handle_send_file`。主要需要组织一些header(http), 然后使用`Body::wrap_stream(reader)`填写文件数据
        + 然后就是各种method的处理了: `match method {}`跳转到各种handler
- API: 只关注几个比较意想不到的
    * `handle_zip_dir`: 文件夹以zip压缩包形式下载
        1. 使用`let (mut writer, reader) = tokio::io::duplex`创建channel类似的东西, writer写reader读
        2. 用ZipFileWriter(writer)包装, 以dfs的方式遍历目录WalkDir记录子目录
        3. 打开并将每个子目录文件写入write, reader读取做response的body
    * `handle_search_dir`: 搜索路径
        1. 展开所有子文件的路径, 软链接等
        2. 不区分大小写的查找, 全转小写后找contain, 发送找到的结果
    * `handle_move`: 移动文件 **`fs::rename()`**
    * `handle_upload`: 上传文件, 拿到请求的数据(body), 转换为`StreamReader::new(body)`, 写入文件
- 页面处理
    * html: 主要显示在`index-page` -> `<tbody>`, 每个文件及其操作都是一个`<td>`标签, 有js负责插入
    * js: `renderPathsTableBody()`插入html标签, 当有数据时移除hidden以显示内容
        + 对于主要的文件下载功能就是一个`<a href="${url}></a>"`标签



### 网络编程

> `create_addr_incoming()`


## rust misc

### 怎么做log

> server的逻辑就一条请求一个log呗

本质上相当于插入一个`vector<log>`数组中


### 通过unix socket链接


## todo

- MakeService是什么设计模式??


