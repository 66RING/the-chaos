#include <format>
#include <iostream>
#include <source_location>

namespace common {

// copy from minilog
template <typename T>
struct with_source_location {
public:
  T fmt;
  std::source_location loc;
public:
  template <typename U>
  consteval with_source_location(U fmt, std::source_location loc = std::source_location::current()): fmt(std::forward<T>(fmt)), loc(std::move(loc)) {}
  std::source_location const location() { return loc; }
  T const format() { return fmt; }
};

template<typename... _Args>
void panic(with_source_location<std::format_string<_Args...>> __fmt, _Args&&... __args) {
  auto loc = __fmt.location();
  std::string user_msg = std::vformat(__fmt.format().get(), std::make_format_args(__args...));
  std::cout << "\E[31;1m" + std::format("[panic] {}:{} {}", loc.file_name(), loc.line(), user_msg) + "\E[m\n";
  exit(1);
}

} // common
