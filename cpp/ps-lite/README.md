# ps-lite

https://zhuanlan.zhihu.com/p/467650462

## roadmap

> 搞清楚数据是怎么在worker, server, postoffice, van, scheduler中流动就行

- 数据流动
    * **只有scheduler会处理控制指令**, 然后通过Send直接发送回去。而在做数据转发的时候则是通过worker/server的Customer代理接收
    * scheduler负责更新server和worker的sender列表使得他们可以直接相互通信
    * abs
        + 每个worker和server都有两个线程一个负责发送一个负责接收，其中负责接收的线程叫做customer
        + 每个worker和server都有自己的一个邮局，里面与其有连接的节点的id
        + worker/server加入到系统会向scheduler发送一个控制信号`ADD_NODE`，scheduler收到后就会广播给系统的其他节点，让他们将新加入的节点添加到自己的通讯录中
- Q: scheduler怎么建立自己的sender的? 即发送给其他节点的入口
    * A: scheduler也是同样的van循环, 然后会收到`ADD_NODE`进入`ProcessAddNodeCommandAtScheduler`，从而建立senders map
- Q: scheduler怎么向worker的customer发送消息? customer记录是什么时候建立的?
    * A: scheduler不转发, 它只是管理布局信息，即有哪些worker和server连接到了系统!

- Postoffice
    * customer list, node list
    * Manage
    * UpdateHeartbeat: TODO
    * Start
    * 增删查customer
- van
    * zmq van: zmq是一个消息队列库
    * zmqvan:Start() -> Van::Start()
        + 解析ip:port, 解析当前node信息 
        + NOTE: 获取scheduler信息: ip等 NOTE
        + 只用zmq API绑定端口
        + 连接到scheduler, zmq API连接, **创建向scheduler发送用的sender**
        + 启动receiver thread做Van::Receiving
    * Send -> SendMsg(zmq_van.h)
        + 填充数据包元信息: 消息类型(pull/push), 请求还是回复等
        + 发送元信息
            + `zmq_msg_init_data`序列化?
            + `zmq_msg_send`发送
        + 发送数据信息
            + `msg.data`
            + `zmq_msg_init_data`序列化?
            + `zmq_msg_send`发送
    * Receiving: 一个接收+处理循环
        + TODO:
        + 数据接收: RecvMsg, 与Send对应
            + i=0时解析发送者id
            + i=1时处理meta data
            + else 处理消息数据
        + 数据处理:
- customer
    * worker和server都可以雇佣customer取邮局取消息, customer是邮局的customer
- worker
- server
- helper class
    * 环境变量管理器

邮局as集散中心, 控制消息传递到哪里。 邮局通过van传递, van传递给worker/server的代理: customer

## 1

查看`tests/test_kv_app_multi_workers.cc`

- client
- server
    * 注册对消息的处理真正的处理逻辑发生在KVServerDefaultHandle
    * `server->set_request_handle(KVServerDefaultHandle<float>());`

- ABS
    * 邮局，邮车，顾客

### 基本使用

看看`test_kv_app`是怎么用的

启动server

- 设置handler
    * default handler会简单遍历worker发送过来的每个kv, 然后对于push请求对key对应的value做累加, 对于pull请求就将累加值返回
- TODO: 哪里的启动?

```cpp
void StartServer() {
  if (!IsServer()) {
    return;
  }
  auto server = new KVServer<float>(0);
  // 注册对消息的处理真正的处理逻辑发生在KVServerDefaultHandle
  server->set_request_handle(KVServerDefaultHandle<float>());
  RegisterExitCallback([server](){ delete server; });
}
```

client端(worker)端

- RunWorker单独线程(demo版): 向server push随机数，然后pull sum值
    * 创建worker: KVWorker

允许测试, `cs tests`

```bash
export PS_VERBOSE=1; ./local.sh 1 1 ./test_connection
```


### 基本组件

- 基本角色
    * worker
    * server
        + 一个kv数据库
        + 处理worker的pull/push请求
            + 处理push请求时有多种变种: 如SGD更新梯度
    * scheduler: 一个系统中只有一个scheduler节点
        + 负责管理worker和server
        + 管理数据同步
- 基本标识
    * node id
        + node id和ip一一对应, 通过`ADD_NODE`指令由scheduler统一分配node id
    * node group
        + 用于代表所有server所有worker节点等
    * app id
        + 创建server只需要app id
        + 创建worker需要app id+customer id, 一个app可以对应多个server
            + GetCustomer先根据app id找到对应的所有customer, 再根据customer id找
        + 考虑ps是一个通用的ps
- **基本模型/抽象**: 经典中的经典
    * PostOffice: 邮局, 消息的集散中心, 管理消息发送给谁(它掌管着在它那里等消息的“顾客”的名单)
    * Van: 邮车, 负责发送消息
    * Customer: 邮局的顾客the 接信员, 负责等待消息
    * Worker: Customer的顾主
    * Server: Customer的顾主
- 辅助类
    * SArray: 智能指针, TODO: review
        + 当引用为0时，能够自动回收内存
        + 实现零拷贝，以提高传递内容时的效率 TODO
    * KVPairs: 用于承载worker/server之间要传递的数据
        + TODO: 一次发送一批
        + `SArray<Key> keys`, key其实就是int64，表明一次通讯中传递了哪些key
        + `SArray<Val> vals`, 表明一次通讯中要传递的值, 如weight, embedding
        + `SArray<int> lens`, 表示每个key对应的value的长度
    * Meta: 通讯信息的元数据
        + sender & recver: 通讯双方的node id
        + app_id & customer_id: 即使在一个node上，也可能有多个server和worker，node会根据app_id+customer_id来找到具体的信息接收方
        + bool request & response: 别人发来的request，还是别人对我的response
        + bool pull & push: requst类型
    * Control: 内部控制指令
        + 控制scheduler和server和worker的交互
        + `barrier_group`: 标识在哪组节点内部需要同步
        + `vector<Node> node`: 当有节点加入或退出时
    * Message: worker/server之间通讯的信息
        + 包含Meta，如前所述，是元数据
        + `vec<SArray<char>> data`，长度是2或3，和KVPairs的转化关系如下：
        ```cpp
        KVPairs<Val> kvs;
        kvs.keys = msg.data[0];
        kvs.vals = msg.data[1];
        if (msg.data.size() > (size_t)2) {
          kvs.lens = msg.data[2];
        }
        ```


## 邮局和邮车

> https://zhuanlan.zhihu.com/p/467693949

### 邮局PostOffice

- abs
    * 异步通信
    * 条件变量


- PostOffice: 全局单例
    * `Van* van`: 负责对外通信, 一个邮局只有一辆邮车（Van）
    * `map<int, vec<int>> node_ids`: group id到node ids，用于查找worker节点
    * `map<int, map<int, Customer>> customers_`: <app id, <customer id, customers>>
        + 接/发线员
    * `map<int, map<int,bool>> barrier_done_`: <app id, <customer id, 同步是否完成>>
    * `vector<Range> server_key_ranges_`: 当前server中存储的key的范围，worker需要找到对应的server
- `PostOffice::Start()`
    * init env
    * init group
    * van start
    * if do sync
- PostOffice中的同步: 多节点之间的同步, 如等待SGD收集完权重
    * `PostOffice:Barrier`: 先向scheduler发送barrier请求，再阻塞等待scheduler的回复
    * `PostOffice:Manage`: 在收到scheduler“同步完成”的信息后，调用PostOffice:Manage函数进行处理
    * TODO
- Customer管理: 通过app id和customer id来定位一个customer


### van通信模块

- van.h基类定义了收发消息的接口, 具体的IO由子类完成
    * 具体使用哪个子类由Postoffice::InitEnvironment()中对Van::Create的调用完成
        + TODO: design
- van启动: `PostOffice->Start`
    1. 和scheduler建立联接 
    2. 启动线程处理接受到的消息
    3. 向scheduler发送ADD_NODE信息注册自己
    4. TODO: review
- Send发送消息
- Receiving接收消息: **接收 + 处理循环**
    * van->Start中，创建了一个独立的线程运行Van::Receiving
        + **处理控制执行 + 处理数据指令**
        + TODO: review
        + ProcessBarrierCommand: 处理同步
        + ProcessAddNodeCommand: 集群内节点管理
        + ProcessAddNodeCommand: 处理add node
        + ProcessDataMsg：处理数据消息


## 顾客、工人和服务器

> customer, worker, server

TODO: review design

Customer从Van手中接过信件后，转手就交给它的顾主，worker或server

- 重要组成
    * `ThreadsafePQueue recv_queue_ + recv_thread_`接收队列 + 处理线程
    * `recv_handle_`: 由创建Customer的雇主(worker/server)指定的消息处理函数
        + 用户自定义处理方法
        + TODO: design
    * `vec<pair<int, int>> tracker_`: 记录每个request的完成情况
        + `pair[0]`记录一共向多少节点发出了请求，`pair[1]`记录多少节点已经回复。当二者相等时，代表请求完成
        + `tracker_`的下标相当于`request_id`，也是NewRequest函数的返回值
- 构造
    * app id + customer id
        + app id 表明这个customer对哪类消息感兴趣
        + 一个worker node可能包含多个worker thread，它们各自有独立的customer_id
    * `recv_handle`，由该customer的雇主传入
    * `Postofficer->AddCustomer`将自己向邮局注册
    * 启动`recv_thread_`
- **API** TODO: how to abs
    * 消息相关API
        + Accept：接受消息
        + Receiving：处理消息
    * 请求相关API: TODO review
        + NewRequest: 在KVWorker生成pull请求时被调用，在tracker_中占一位，并且设置“请求节点总数”
        + WaitRequest: 条件变量阻塞等待“请求节点数”和“回复节点数”相等。每次Receiving都会触发一下





















