# 为什么这样不行

```cpp
template <typename T>
struct with_source_location {
private:
  std::source_location loc;
  T fmt;
public:
  // HERE T
  consteval with_source_location(T &&fmt, std::source_location loc = std::source_location::current()): fmt(std::forward<T>(fmt)), loc(std::move(loc)) {}
  T format() { return fmt; };
  std::source_location location() { return loc; }
};


template <typename ..._Args>
void log_info(with_source_location<std::format_string<_Args...>> __fmt, _Args&&... __args) {
  auto const &loc = __fmt.location();
  std::cout << std::vformat(__fmt.format().get(), std::make_format_args(__args...));
}


int main() {
  log_info("{}", "info");

}
```

提示`const char*`不能转换成`with_source_location<std::basic_formatstring<>`

而这样可以

```cpp
template <typename T>
struct with_source_location {
private:
  std::source_location loc;
  T fmt;
public:
  // HERE U
  template <class U>
  consteval with_source_location(U &&fmt, std::source_location loc = std::source_location::current()): fmt(std::forward<U>(fmt)), loc(std::move(loc)) {}
  T format() { return fmt; };
  std::source_location location() { return loc; }
};

template <typename ..._Args>
void log_info(with_source_location<std::format_string<_Args...>> __fmt, _Args&&... __args) {
  auto const &loc = __fmt.location();
  std::cout << std::vformat(__fmt.format().get(), std::make_format_args(__args...));
}

int main() {
  log_info("{}", "info");
}
```

因为如果不用U, 那特化的`with_source_location`的内部类型是`std::format_string<_Args...>`, 且构造用的也是`std::format_string<_Args...>`, 所以不能用`char*`

