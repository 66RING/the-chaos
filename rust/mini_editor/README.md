# 终端编辑器的简单实现

> [ref](https://medium.com/@otukof/build-your-text-editor-with-rust-678a463f968b)

- 怎么显示
    1. 开启raw mode, 使用手动"回显"
    2. 批量组织内存中的数据(buffer), 包括文本, ANSI控制转意命令等
    3. 记录窗口信息, 长宽, 偏移等
    4. 每个事件渲染一下屏幕, 即将buffer中的内容写到stdout中
- 怎么设置光标
    * 使用ANSI conole控制指令: 转意输出到stdout
    * 可以控制移动, 显示隐藏等
- 怎么改动文本: 增删
    * 删: 用`Vec<line>`保存每行, 定位到行后定位到删除的位置即可
    * 增: 同删, x, y定位后String.insert
- 怎么走两套输出模式, 如普通输入时输入显示在正文窗口, 保存文件时输入显示在状态栏窗口, 如输入保存文件名
    * 使用多个状态机: 看到的数据的相同的(复用), 但做出的行为不一样
    * 多状态机可以使用rust的宏功能自动生成
- 总结一下整体结构设计
    * TODO
    * raw mode


- misc
    * raw mode是干嘛的


1. setup
2. reading keypress and entering “raw mode”,
3. drawing on the screen and moving the cursor around,
4. displaying text files (making our program a text view),
5. editing text files,
6. saving changes
7. implementing a cool search feature and finally to adding syntax highlighting


## 1. setup

```toml
[package]
name = "pound"
version = "0.1.0"
edition = "2021"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
crossterm = "0.21.0" #add this dependency
```

## 2. reading keypress and entering “raw mode”,

- 使用`io::stdin::read()`读取输入
- 默认终端会启动在[canonical mode](https://en.wikipedia.org/wiki/POSIX_terminal_interface#Canonical_mode_processing)
    * 这个模式会回显你的输入, 然后在你按下enter时"定格"
- 启动raw mode, 使用`crossterm`这个crate
    * 因为我们不想终端自动回显一些奇怪的东西
    * 并注意退出时关闭raw mode, 实现Drop trait
    * raw mode中需要手动使用`\r`回车
- 处理键盘事件: 普通key, 退格, pageup等
    * while loop, 然后使用crossterm::event::read()来match
- 太久没输入的超时机制可以通过`crossterm::event::poll`实现


## 3. drawing on the screen and moving the cursor around,

- abs
    * ANSI console控制
        + 清屏
        + 光标位置, 显示隐藏

- 重构键盘输入, 抽象一个Reader, 封装超时和read, 返回一个event
- 创建Editor对象, 作为APP主程序, 它应该包含一个reader
    * 创建run主函数: read, handle
- render: 从清屏开始, 每次启动都应该"清屏"
    * 使用ANSI console控制指令清屏, 如`print!("\x1b[2J");stdout().flush();`
- 调整光标位置在左上角
    * 同样是使用console的控制指令移动光标
- 退出时清屏, 不留app痕迹
- eof后的每行前都加上`~`符号, 像vim一样
    * 清屏, move(0, 0), 画行, move(0, 0)
- 描述窗口对象, Output
- 批量IO, buffer: EditorContents结构
    * 记录内容和ANSI指令, 可以用String::push填充
    * 实现io::Write trait来实现对Buffer的写入和回显
    * 使用`crossterm::queue!(dyn Write, content, ...)`批量写入
- 渲染时隐藏鼠标
    * cursor::Hide/Show
- 渲染优化
    * 每次刷新不再整个屏幕刷新, 而是只刷新修改的地方
- 渲染问题
    * 判断窗口边界, 必要时简单阶段
- 光标控制, 如上下左右移动, CursorController抽象
    * 记录光标位置
    * 处理按键事件
    * 刷新屏幕时光标移动到记录的位置
- 防止光标越界
    * 光标控制器中记录窗口大小信息
- 其他按键处理: PageUp, PageDown, Home, End等


## 4. displaying text files (making our program a text view),

- abs
    * backend文件数据EditorRows
    * 画面滚动

- 抽象每行数据EditorRows
    * EditorRows中的数据可以逐行push到buffer中
- EditorRows中保存的是实际文件的内容
    * new时读取并填入
- 画面滚动
    * CursorController中记录基础的offset, `draw_rows`时在offset的基础上取
    * 每次refresh都检测scroll情况, 光标移动到边界时更新offset: `scroll`
- 水平滚动
    * 一个水平的offset, 对于字符不够长的行, 直接显示空白
    * 左回环上移, 右回环下移
- 渲染Tabs
    * 抽象每行的数据表示Row
        + 因为raw data(&str)会包含`\t`等不好直接输出的内容
        + 可以保留一个raw data和一个render data
    * 读取`\t`然后翻译成空格, 保存到render data中
- 添加光标和tab的交互, 光标应该将tab看作一个整体
- 更新事件处理的逻辑, 支持滚动等
- 添加状态栏
    * 渲染正文时预留底下一行
    * 记录文件的一些元信息, 如使用PathBuf解析文件名
        + 修改时间, 初始信息等


## 5. editing text files,

- 修改Row抽象
    * 然raw data也是String, 这样就可以修改了
- 添加普通按键事件
    * rust tips: `code @ (KeyCode::Char(..) | KeyCode::Tab)` wild card
- 保存到磁盘
    * 根据文件名打开文件, write all
    * 添加保存按键事件`<C-s>`
    * 添加message bar消息更新功能
- 增加Dirty flag等功能
    * dirty: 保存文件后清除标记
    * 退出未保存的提示
- 添加删除功能
    * 定位: 根据当前光标x, y定位
- 处理换行
    * 相当于在`Vec<Row>`中插入一个Row
- 另存为
    * `prompt!()`宏, `<C-s>`时进入另一个状态循环
    * 当然宏可以更通用点, 不局限于另存为的情况
    * Escape取消输入


## 6. search feature

- 进入查找模式
    * 本质和另存为是一样的逻辑, 接收用户的输入`prompt!()`返回用户输入做响应的逻辑
- 遍历Row然后逐一find
- `prompt`宏添加`callback`函数, 以提供自定义功能
    * 如输入搜索后画面动态跳转
- 光标的恢复
    * 实现copy, clone后再进入前拷贝, 退出后考会即可
- 向前搜索/向后搜索
    * 记录一个search index, 可以记录x, y
    * 前向就是向substring的右下查找
    * 后向就是向substring的左上查找

