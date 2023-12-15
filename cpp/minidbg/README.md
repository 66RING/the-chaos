# minidebug

## setup

- abs
    * linenoise库
        + TODO
    * ptrace
        + `PTRACE_CONT`继续执行
        + `PTRACE_TRACEME`允许追踪
    * `waitpid`
    * `fork`


- launch debugger with fork/exec
- 基本框架
    * **子进程**中启动被调试程序
        + 使用`ptrace`可以控制其他进程的执行: 控制寄存器、内存、单步调试等
    * **父进程**与用户交互(读入用户输入)

```c
long ptrace(enum __ptrace_request request, pid_t pid,
            void *addr, void *data);
```

- `request`对目标进程要做的操作
    * `PTRACE_TRACEME`允许父进程追踪
- `pid`目标进程id
- `addr`一些操作使用，用于指定被调试进程的地址
- `data`一些request需要的数据


## set breakpoint

- abs
    * 小端存储
    * 8byte改1byte并注意小端存储
    * ptrace(读写都是8byte为单位)
        + 读`PTRACE_PEEKDATA`
        + 写`PTRACE_POKEDATA`

- 两类breakpoint
    * 硬件: 通过设置硬件的寄存器触发
        + 提供了在读写地址时触发的能力
    * 软件: 修改原始代码触发(如插入中断语句)
        + 只有执行到才会触发

- 软件断点设置(修改源码)
    * 如何修改
        + 使用`ptrace`可以读写内存，从而修改
    * 怎样的修改
        + 想办法让程序halt然后发信号给debugger: x86中可以使用int 3指令(`0xcc`)
        + 程序执行到int 3指令时，会通过中断表查到执行breakpoint interrupt handler。在linux下，它会向进程发送`SIGTRAP`信号
    * 如何和debugger交互
        + `waitpid()`等待信号(sigtrap或者其他)

- 插入int 3指令, 一个字节的0xcc
    * 保存原始指令
        + 读取目标地址处8byte的数据`ptrace(PTRACE_PEEKDATA, m_pid, m_addr, nullptr)`
        + 需要注意的是一次读取8byte
    * 插入0xcc
        + 将`0xcc`替换原始数据的**小端**
        + 使用`ptrace(PTRACE_POKEDATA, m_pid, m_addr, data_with_int3);`写回
        + 需要注意的是一次写入8byte, 并且小端存储
    * 恢复程序执行

- 整合debugger类
    * 暴露API: `set_breakpoint_at_address(addr)`
    * 记录所有断点: `map<addr, breakpoint>`
    * 添加`break`指令: 判断输入是break开头, 并获取addr参数`stol`转换成地址

- 从断点恢复
    * **一个简单的处理(因为后续需要更新)**: 关闭断点, 单步执行, 重启断点, continue

- 测试
    * 目前直接使用地址设置断点的可用性还太差, 后面需要添加根据函数名设置断点
    * 这里简单起见可以通过`objdump -d <prog>`查看地址偏移
        + 不过这样看到的地址可能不是绝对地址，可能会收到position independent executable和address space layout randomization的影响
        + 使用`-no-pie`编译并在执行目标程序前调用`personality`关闭地址随机化: `personality(ADDR_NO_RANDOMIZE);`
    * 查看程序加载地址
        + 通过子进程ip查看`/proc/<pid>/maps`里的地址映射情况
        + 找到base address在加上objdump看到的偏移

## TODO

- rewrite it in rust!!!!!!!










