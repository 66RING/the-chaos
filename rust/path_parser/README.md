# path parser

简单的`url`parser, 体会parser的设计。

- abs
    * url路径抽象成iter
    * `next`

## impl

url格式`/a/b/c`, 整体流程抽象如下:

1. 消耗`/`得`a/b/c`
2. 根据下一个`/`split得到当前路径`a`和剩余路径`/b/c`

