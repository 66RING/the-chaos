# parser

一个parser的本质是模式匹配, 有两层含义:

1. 字面的字符匹配
2. 一种模式是固定的, 所以做处理时可以抽象出定量再递归匹配变量

比匹配一个`(a+b)`就可以先把固定的`()`写在handler中。

常见的抽象是:

1. 游标表示当前索引位置
2. line, line position等作为报错辅助信息
3. `peek`, 获取当前字符因为有的场景需要特判
5. `get/consume`, 消耗当前字符, 并将游标后移
6. `inc`, 游标后移
7. `eat_space`, 消耗后续所有空白符 
8. `eat_char(c)`, 消耗后一个char并判断是否与c相同



