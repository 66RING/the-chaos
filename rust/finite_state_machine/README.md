# 正则库/有限状态机

> 用fsm实现一个简单的正则表达式库
>
> ref [Regex Tsoding Session](https://www.youtube.com/watch?v=MH56D5M9xSQ).

!!!!!!WIP!!!!!!!!

TODO:

- 搞清楚`.*`后存在其他字符串的case即可, 怎么防止`.*`不会吞掉所有, e.g. `.*bc`
- 搞清楚`.*bcbc`如何贪婪匹配且保证后续bcbc存在
- 我觉得要实现贪婪模式是需要回溯的


## 重点

着重理解`*`的实现

## 使用

一个字符串"abc", 其中每个字符都是一个state


- 优化
    * 基数树


## 实现

- 组成
    * State
    * Event
        + 入, 执行, 出
    * **Trans**: `next_state(curr_state, event) -> new state`

对于复杂而大量的状态和处理, 人们常常使用自动生成的方式生成状态机, 而不是手动在代码中实现每个。这可以通过一个二维数组表示state和trans的关系, 而二维数组中记录的就是下一个状态的id。(as 编译原理学的)。自动化生成这么一个二维向量(表)就简单多了。

- 二维表实现: 两个vector, `vector<Column>`, 然后每个Column结构中记录的其实是transition

```rust
struct Column {
    transition: Vec<usize>,
}

type FSM = Vec<Column>
```

- 二维表初始化: range填充
    * 特殊状态, e.g. `abc$`
- API: regex
    * **`compile`**: 字符串转换成fsm
        + 需要重点考虑分支和回环(**L2**)的情况, `+.*|`
            + **`*`**: `*`的前一个字符可以出现或者出现任意次
                - `a*b`体现在状态机上就是, 如果前一个状态不定出现则不断回到前直到遇到b或者其他东西
                - 而对于遇到的其他字符则可以跳到下一个stage, 然后要匹配的字符串轮空
            + `+`: `+`的前一个字符出现至少一次, 即是`a+ => aa*`
            + `.`, 下一个匹配所有
    * **match** -> bool
        + 怎么才算match? 沿着状态机stage走:
            - 走到了0(invalid)就是没有match
            - 如果没走完所有stage, 且end stage的标志位也没置位也没match, 即查看当前stage是否是一个有效的end stage
    * dump: 打印二维表

## TODO

- 想办法实现group: `()`
    * hint: 递归, 组内的失败不会让整个fsm失败


