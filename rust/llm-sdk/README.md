# llm-sdk

> [ref](https://github.com/tyrchen/llm-sdk)

## create image

### project

- TODO: 学习一下
    * clippy: `cargo clippy`, 更加rust
    * pre-commit-config

### 抽象

- 通过结构体生成request(builder模式)
    * 使用into抽象
    * TODO
- 定义好的东西, 如果有限可穷举, 我们最好都是用enum定义
- 设计模式
    * builder模式, 一个结构体的成员很多, 一个一个填充很麻烦的时候

### crate

- `derive_builder`
    - 一个结构内有很多参数, 自己构建new函数再一个一个赋值太麻烦了。可以使用`derive_builder`自动构建, 我们只需要描述好结构体就可以了。
- `serde`
    * 各种serde技巧, 自动serde json的, 跳过的, 默认的, rename的
    * 自动snake case等


## chat completion

### crate

- serde enum tag
    * enum名自动添加成一个字段


