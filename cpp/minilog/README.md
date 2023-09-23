# log库

> [minilog](https://github.com/archibate/minilog)

- abs
    * X Macro
    * 怎么封装成单一头文件, ODR
    * static initialization技巧

## impl

- 使用`#include <source_location>`这个库, 而不使用`__FUNCTION__`之类的宏。`source_location`有`file_name()`返回文件名, `line()`返回行号等功能。
    * 用法`loc = std::source_location::current()`返回一个`source_location`对象, 内部记录的调用时所在的文件和行号
- 思路log库的API一般就是一个函数加format的形式, 然后自动增加文件信息, 如: `info("format {}", "...")`
    * 这里有个tips是, `loc = std::source_location::current()`作为参数默认值时, 记录的位置是实际调用的位置
- 等级系统
    * 使用`enum class`枚举, 数值越大越要报告, 这就实现了过滤
    * 枚举名字获取: X Macro
- 自动打印变量值: X Macro技巧
    * `#define MINILOG_P(x) ::minilog::log_debug(#x "={}", x)`
    * 字符串自动拼接: `"hi" " there"`自动转化成`"hi there"`
- 从环境变量获取: `std::getenv("VAR")`
    * 使用static initialization的技巧让全局变量的构造方法更加灵活 
- **封装成头文件**
    * 加名字空间
    * 开头`#pragma once`防止用户重复include**头文件**
    * **解决ODR问题: 唯一定义原则**, 因为我们的库的单头文件的库, 所以用到的函数直接就定义在头文件里了, 头文件多次展开后产生多个定义了
        1. 模板函数会自动添加`inline`, 所以OK
        2. 不是模板函数就手动给函数加inline和给变量加static
            - 需要注意头文件中的static变量在各个cpp文件中不共享, 所以在c++17后加inline的变量就可以共享了
    * **宏中的名字空间**, 因为宏是原地展开的嘛, 所以如果在宏中使用了当前命名空间的函数则在展开时可能其他的同名函数, 所以在使用`::`开头的名字空间`::my_namespace`先跳到全局的名字空间以确保路径正确
    * 封装内部函数，在用一层名字空间封装内部使用的函数, 这样用户接口更清晰
- 彩色文字: 一些ANSI的终端控制的特殊输入
- 原子性: 是并发程序下到处打log的场景
    * 一次输出符号`<<`可以保证原子性, 所以在`<<`输出前format好, 而不是用`<<`拼接


## misc

### static initialization

构造全局变量, 但变量构建的过程可能存在许多分支(如不存在时使用默认值)。本质就是定义一个匿名函数然后立刻调用它。

```cpp
inline log_level g_max_level = [] () -> log_level {
    if (auto lev = std::getenv("MINILOG_LEVEL")) {
        return details::log_level_from_name(lev);
    }
    return log_level::info;
} ();
```

### X Macro技巧

> [知乎教程](https://zhuanlan.zhihu.com/p/521073931)

1. 小范围内定义宏, 然后释放(undef)
2. 本质就是利用宏函数的一些功能来实现诸如字符串拼接, 转字面值, 转"值"等操作
    - `#define X_MACRO(x) x,`, 原样输出并添加一个都好
    - `#define X_MACRO(x) #x`, 输出字符串字面值"x"
    - `#define X_MACRO(x) log_##x`, 拼接字符串
    - 字符串自动拼接: `"hi" " there"`自动转化成`"hi there"`

举个例子, 自动生成模式(由`_FUNCTION`定义模式), 自动生产switch case等:

```cpp
#define XMACROS_TABLE(f) \
	f(trace) \
    f(debug) \
    f(info) \
    f(critical) \
    f(warn) \
    f(error) \
    f(fatal)

enum class log_level : std::uint8_t {
// X Macro原样返回name, 快速完成定义
#define _FUNCTION(name) name,
    XMACROS_TABLE(_FUNCTION)
#undef _FUNCTION
};

inline std::string log_level_name(log_level lev) {
    switch (lev) {
// X Macro批量生成 case log_level::name: return #name
#define _FUNCTION(name) case log_level::name: return #name;
    XMACROS_TABLE(_FUNCTION)
#undef _FUNCTION
    }
    return "unknown";
}

int main() {
	std::cout << log_level_name(log_level::info) << "\n";
}
```


### 结构体隐式构造

say需要一个A类型的参数, 但是我们使用时传入的是int, 这是因为发生了隐式构造。这样我们就可以方便的通过隐式构造使用函数, 而在函数内使用构造好的结构体使用oop。

```cpp
struct A {
public:
	std::string msg;
	A(std::string n): msg(n) {}
};

void say(A a) {
  std::cout << a.msg << std::endl;
}

int main() {
  say(std::string("hi"));
  return 0;
}
```

更复杂一点可以是:

```cpp
struct A {
public:
	std::string msg;
	int id;
	A(std::string n, int a): msg(n), id(a) {}
};

void say(A a, std::string n) {
  std::cout << a.msg << ":" << a.id << " " << n << std::endl;
}

int main() {
  say({std::string("hi"), 10}, "ring");
  return 0;
}
```

### c++17后的临时变量if

```cpp
auto tmp = get();
if (tmp == 4321)
    return true;
return false;
```

可以简写成, 防止临时变量泄漏和污染

```cpp
if (auto tmp = get(); tmp == 4321)
    return true;
return false;
```


