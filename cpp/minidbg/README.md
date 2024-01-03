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


## 控制寄存器和内存

- abs
    * ptrace
        + `PTRACE_GETREGS`获取寄存器array
        + `PTRACE_SETREGS`写入寄存器array
    * offset map: linux里一些API都是返回`void*`的,方便起见可以构建一个offset map
    * lib
        + `process_vm_writev`, `process_vm_readv`

- 根据x86的spec一个寄存器需要他的名字和DWARF register number
    * 寄存器结构体可以查看`/usr/include/sys/user.h`
    * DWARF register number可以查看[System V x86_64 ABI](https://www.uclibc.org/docs/psABI-x86_64.pdf)

- 读取寄存器
    * ptrace的`PTRACE_GETREGS`指令, dump出一个寄存器对象(`uint64_t`数组): `ptrace(PTRACE_GETREGS, pid, nullptr, &regs)`
    * 访问指定位置寄存器: 创建一个寄存器偏移表, 查表获取偏移来访问array
- 写入寄存器
    * `PTRACE_SETREGS`
    * 整个寄存器的array写回: `PTRACE_GETREGS`读一份array, 修改一点, 再写回
- 通过DWARF register number查找寄存器
    * 同理, 建立一个偏移查找表
- 添加读取寄存器的命令: `register {dump|read|write}`
    * `dump_registers`helper函数便利每个寄存器的数据
- 读取内存
    * 封装`PTRACE_PEEKDATA`, `PTRACE_POKEDATA`做内存读写
    * 一次读写大块内存可以考虑继续使用ptrace或者使用[`process_vm_readv`和`process_vm_writev`](http://man7.org/linux/man-pages/man2/process_vm_readv.2.html)
- 添加内存访问指令
- 完善continue
    * 因为已经可以读取寄存器了, 所以再如果要单步执行跳过断点可以通过检查pc指针是否有bp, 然后关闭, 单步, 重启。


## Elves and dwarves(ELF and DWARF)

> ELF: Executable and Linkable Format
>
> DWARF: Debug information format most commonly used with ELF

- abs
    * **像解析ELF一样解析DWARF**

- 调试信息格式: DWARF
    * 和ELF类似`-g`编译后会在二进制文件中嵌入调试信息，如文件行号再内存的哪个位置
    * 使用`dwarfdump`工具可以查看调试信息
    * DIE(Debugger info entry)

dwarfdump格式如下，

```
.debug_line: line number info for a single cu
Source lines (from CU-DIE at .debug_info offset 0x0000000b):

            NS new statement, BB new basic block, ET end of text sequence
            PE prologue end, EB epilogue begin
            IS=val ISA number, DI=val discriminator value
<pc>        [lno,col] NS BB ET PE EB IS= DI= uri: "filepath"
0x00400670  [   1, 0] NS uri: "/path/to/test.cpp"
0x00400676  [   2,10] NS PE
0x0040067e  [   3,10] NS
0x00400686  [   4,14] NS
0x0040068a  [   4,16]
0x0040068e  [   4,10]
0x00400692  [   5, 7] NS
0x0040069a  [   6, 1] NS
0x0040069c  [   6, 1] NS ET
```

如何根据DWARF使用行好设置断点: 根据行号查找entry, 查看地址如这里是最后一个line3： `0x0040067e`。这个地址是从加载地址开始的偏移。

**`.debug_info`块**

```
.debug_info

COMPILE_UNIT<header overall offset = 0x00000000>:
< 0><0x0000000b>  DW_TAG_compile_unit
                    DW_AT_producer              clang version 3.9.1 (tags/RELEASE_391/final)
                    DW_AT_language              DW_LANG_C_plus_plus
                    DW_AT_name                  /path/to/variable.cpp
                    DW_AT_stmt_list             0x00000000
                    DW_AT_comp_dir              /path/to
                    DW_AT_low_pc                0x00400670
                    DW_AT_high_pc               0x0040069c

LOCAL_SYMBOLS:
< 1><0x0000002e>    DW_TAG_subprogram
                      DW_AT_low_pc                0x00400670
                      DW_AT_high_pc               0x0040069c
                      DW_AT_frame_base            DW_OP_reg6
                      DW_AT_name                  main
                      DW_AT_decl_file             0x00000001 /path/to/variable.cpp
                      DW_AT_decl_line             0x00000001
                      DW_AT_type                  <0x00000077>
                      DW_AT_external              yes(1)
< 2><0x0000004c>      DW_TAG_variable
                        DW_AT_location              DW_OP_fbreg -8
                        DW_AT_name                  a
                        DW_AT_decl_file             0x00000001 /path/to/variable.cpp
                        DW_AT_decl_line             0x00000002
                        DW_AT_type                  <0x0000007e>o...
```

根据pc判断所处函数/变量

```
根据顶层compile unit(CU)判断pc是否在这个范围
    根据子层compile unit(CU)判断pc是否在这个范围
```

根据函数名设置断点: 

1. 遍历每个CU的`DW_AT_name`或者查看`.debug_pubnames`段找到到所在table
2. 再根据table记录的信息跳过一些无关的地址

根据变量名读取值

1. 根据`DW_AT_name`查找变量名找到对应CU
2. 根据CU内的`DW_AT_location`字段获取变量再栈帧内的偏移
3. 栈帧基地址可以通过`DW_AT_frame_base`获取


## DWARF解析

- abs
    * TODO: 第三方库的编译和使用: libelfin
    * `/proc/<pid>/maps`

- `DWARF`解析可以使用`libelfin`库
    * 读取elf, 然后根据elf读取到dwarf块
- 利用`libelfin`库实现根据pc找entry和找function的功能
- 通过`/proc/<pid>/maps`查找地址映射
- 打印源码
    * 断点触发后打印一行告诉我们断在哪里了
    * 读取源码文件, 然后getline即可
- 改善信号处理: 回显触发的哪个信息
    * 根据ptrace的spec定义一个`siginfo_t`结构体来解析信号信息
- 处理trap信号
    * 断点触发时会发送`SI_KERNEL`或`TRAP_BRKPT`。单步执行完成后会触发`TRAP_TRACE`
    * 详见`man sigaction`
    * 完善了trap处理后之前根据pc做的单步跳过就可以改掉了

- **Cmake工程**
    * 添加libelfin库到项目
        1. 从源码编译目标库: `add_custom_target(libelfin COMMAND make WORKING_DIRECTORY ${PROJECT_SOURCE_DIR}/ext/libelfin)`
        2. 将编译好的库连接到本项目:
        ```cmake
        target_link_libraries(minidbg
                              ${PROJECT_SOURCE_DIR}/ext/libelfin/dwarf/libdwarf++.so
                              ${PROJECT_SOURCE_DIR}/ext/libelfin/elf/libelf++.so)
        ```
        3. 告知cmake存在依赖关系: `add_dependencies(minidbg libelfin)`
    * 添加目标测试程序的编译
    ```cmake
    add_executable(variable examples/variable.cpp)
    set_target_properties(variable
                          PROPERTIES COMPILE_FLAGS "-gdwarf-2 -O0")
    ```



## source level stepping

- abs
    * ptrace
        + `PTRACE_SINGLESTEP`
    * 单步调试的本质是设置一个临时断点, step后删除

- 支持单步调试, 添加`stepi`指令
- step out, 相当于finish
    * 获取函数的返回地址, 然后再这个地址打断点并执行
- step in
- **step over**, 单行执行, 本质也是设置一个临时断点
    * 难点在于直接根据源码的下一行设置断点是不行的, 因为源码的下一行是哪里无法确定(可能是空行，可能是分支)
    * 真实的debugg会根据当前指令也做分支判断断点设置再哪
    * **这里的简单实现是: **给当前函数后面的每一行都设置断点, step后再移除



## TODO

- rewrite it in rust!!!!!!!










