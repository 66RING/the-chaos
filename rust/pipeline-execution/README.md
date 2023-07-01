# Naive Pipeline Execution (Toy)

> [知乎](https://zhuanlan.zhihu.com/p/614907875) to summary pipeline.

> 本质就是: 数据到位, 通知就绪


## Insight

> 计算任务抽象成DAG, pipeline需要调度节点(任务)的执行

- 朴素法: 多线程, 能跑就跑, 不能就yield
    * 问题在于OS不知道任务之间的依赖关系, 浪费切换的时间
- pull-based, 从下游节点开始, 请求/查看上游节点执行
- push-based, 从上游节点开始执行, 完成后通知相邻的下游节点执行, 放到任务队列中
- 显式watchdog/scheduler调度, 一个独立线程管理该调度/唤起谁

- work stealing
    * 工作强度不同, 通过获取其他任务队列中的任务来分担
    * **delay scheduling**: 不立刻steal, 而是等上一段时间, 因为执行速度还是稍微有点偏差的。因为还可能有跨越NUMA等问题
    * work stealing不是银弹, 也需要见机行事, 如存在跨越NUMA的情况

## impl


- 结构要素
    * DAG图: 依赖关系, 数据流动
    * 执行器抽象, 或者说是task, 执行者
    * thread pool
    * source(start) -> transform(task) ... -> merge(end)
        + merge操作其实就是把所有branch的输出收集起来, 可以是reduce, concat, 集合操作等
    * 状态机主循环
    * helper
        + 辅助图的索引, 如每层的id
- 简单实现(状态机), push-based
    * 起点任务初始时直接标注Ready, `add_source`
    * 之后每个任务可以看成一种操作, `add_transform`
    * loop不断查找Ready任务到线程池执行
    * 任务执行完成后将下一个任务置为Ready, 自己置为Finished
    * **依赖建立方面**
        + 操作符的抽象? N = reduce => 1, 1 = partition => N, 1 = split => p1, p2等
    * 数据传递方面
        + 建图时可以直接使用共享内存的方式(共享内存的说法更抽象, 可以扩展到分布式场景)将前驱的输出绑定到后继的输入: `connect_from_input`
        + 基于上述依赖建立, 对于简单的pipeline来说数据传输就是直接的向后传递
    * 最后使用一个merge节点保证单一出口, 节点的输出就是结果
- 优化
    * 快速找Ready, 单独Ready queue
- tips
    * 流水线可能会不停流动, 从而导致数据更替, 因此需要注意使用原子操作


## rust tips

- 线程池可以使用rayon::ThreadPool
    * 可以使用`ThreadPoolBuilder::num_threads(n)`创建容量
    * 使用`spawn(f)`压入线程池执行
- arrow::record_batch::RecordBatch
- anyhow::Error

