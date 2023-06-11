# 终端彩色字体的库(unix only)

本质:

- 使用[ANSI转意字符](https://en.wikipedia.org/wiki/ANSI_escape_code)控制终端行为
- 重载`<<`运算符以提供方便的拼接接口
- 一些模板的trick: `enable_if`简化模板实现

```cpp
template <typename T>
using enableStd = typename std::enable_if<
  std::is_same<T, rang::style>::value || std::is_same<T, rang::fg>::value
    || std::is_same<T, rang::bg>::value || std::is_same<T, rang::fgB>::value
    || std::is_same<T, rang::bgB>::value,
  std::ostream &>::type;
```

