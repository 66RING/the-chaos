# 简易向量数据库

- http server
- db存储
- 相似度计算
    * 为什么要normalize
        + 因为余弦相似度的计算公式需要归一化, 归一化后可以复用点乘的算子
- 小顶堆保存top k
    * 只查找相似度最高的top k个
- [相似度是怎么计算的](https://cloud.tencent.com/document/product/1709/95430)
    * [点积相似度、余弦相似度、欧几里得相似度](https://zhuanlan.zhihu.com/p/159244903)
    * TODO:
        + 有几种相似度: 点乘, 欧式, 余弦相似度
        + attention?


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



