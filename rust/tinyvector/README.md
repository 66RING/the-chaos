# 简易向量数据库

> 参考自[tinyvector](https://github.com/m1guelpf/tinyvector)

- http server
- 简易db存储
- 相似度计算
    * 为什么要normalize
        + 因为余弦相似度的计算公式需要归一化, 归一化后可以复用点乘的算子
- 小顶堆保存top k
    * 只查找相似度最高的top k个
- [相似度是怎么计算的](https://cloud.tencent.com/document/product/1709/95430)
    * [点积相似度、余弦相似度、欧几里得相似度](https://zhuanlan.zhihu.com/p/159244903)
    * 有几种相似度: 点乘, 欧式, 余弦相似度
        + TODO: 理解为什么可以这么算
- **原版cosine会将归一化后的vector保存起来, 这样计算能更快, 但是这里只存储原始数据了**


## crate

- thiserror
- **rayon: 数据并执行计算库**
    * `par_iter()`并行迭代器
- enum可以考虑直接derive copy
- ⭐axum
    * layer(Extension())怎么用
        + 好像就是单例的传递。用于状态共享
        + 常会使用一些rc技巧如: arc + lock
    * axum的response处理


## 相似度

- 内积(IP)相似度
    * $a \cdot b = \Sigma {a_i b_i}$
- 欧式距离(L2)
    * $d(a, b) = d(b, a) = \sqrt{\Sigma{(b_i - a_i)^2}}$
- 余弦相似度(COSINE)
    * 计算两个向量在多维空间中的夹角余弦值来衡量它们的相似程度
    * $cos(a, b) = \frac{a \cdot b}{\Vert a \Vert \Vert b \Vert}$
    * 归一化计算方法$\Vert a \Vert = \sqrt{\Sigma{a_i^2}}$


## impl

- Q: 原版tinyvector中为什么一个Collection可以有多个embedding
- A: 因为Collection是一个Table的意思
- Q: 原版tinyvector会将归一化的cosine向量存储起来以便加速
- A: 这里只存储原始向量数据, 这样能更通用

- 启动server
    - 初始化db
    - shutdown: 通知器优雅地关闭
    - ⭐router: api handler
        * layer(Extension), 可以实现状态共享, 让单例在handler间传递
    - 启动server监听端口
- 初始化db: `from_store`
    * 本质及就是一个hashmap, 可以读写到磁盘以做持久化
        + 持久化: 简单的`bincode::deserialize(object)`和`bincode::serialize(self)`
- 数据库实现
    * get: hashmap的get
    * insert: 向table中追加embedding, 需要检查embedding的各种信息判断是否兼容(如维度)
    * delete: hashmap的delete
    * create: 判断是否存在然后创建
- ⭐Collection实现: Query时`query_collection`
    * `get_similarity(&self, query: &[f32], k: usize)`
    * 遍历表(Collection)内每个embedding, 使用比较方法计算score, `distance_fn(query, each)`
    * 使用最小堆保存并返回即可
- `similarity.rs`
    * 保存一些相似度处理的方法
- `routes`
    * 负责处理http请求
- graceful shutdown, 我们需要让程序正常退出已保证数据正常写回数据库
    * 启动一个单独的线程来监听ctrl c和terminate信号, 通过channel通知主循环停止

