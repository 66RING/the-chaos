#pragma once
#include <format>
#include <source_location>
#include <chrono>
#include <iostream>
#include <fstream>
#include <cstdint>

namespace minilog {

#define MINILOG_FOR_EACH(f) \
    f(trace) \
    f(debug) \
    f(info) \
    f(critical) \
    f(warn) \
    f(error) \
    f(fatal)

enum class log_level: uint8_t {
#define _PATTERN(x) x,
  MINILOG_FOR_EACH(_PATTERN)
#undef _PATTERN
};

namespace inner {

inline std::string log_level_name(log_level lev) {
  switch (lev) {
#define _PATTERN(x) case log_level::x: return #x;
  MINILOG_FOR_EACH(_PATTERN)
#undef _PATTERN
  };
	return "unknown";
}

inline log_level log_level_from_name(std::string lev_name) {
  if (lev_name == "info") return log_level::info;
#define _PATTERN(x) if (lev_name == #x) return log_level::x;
  MINILOG_FOR_EACH(_PATTERN)
#undef _PATTERN
  return log_level::info;
}

inline log_level g_max_level = [] () -> log_level {
  if (auto lev = std::getenv("MINILOG_LEVEL"); lev) {
	return log_level_from_name(lev);
  }
  return log_level::info;
} ();

inline std::ofstream g_log_file = [] () -> std::ofstream {
    if (auto path = std::getenv("MINILOG_FILE")) {
        return std::ofstream(path, std::ios::app);
    }
    return std::ofstream();
} ();

inline constexpr char k_level_ansi_colors[(std::uint8_t)log_level::fatal + 1][8] = {
    "\E[37m",
    "\E[35m",
    "\E[32m",
    "\E[34m",
    "\E[33m",
    "\E[31m",
    "\E[31;1m",
};
inline constexpr char k_reset_ansi_color[4] = "\E[m";

template <typename T>
struct with_source_location {
private:
  std::source_location loc;
  T fmt;
public:
  template <class U> requires std::constructible_from<T, U>
  consteval with_source_location(U &&fmt, std::source_location loc = std::source_location::current()): fmt(std::forward<U>(fmt)), loc(std::move(loc)) {}
  T format() { return fmt; };
  std::source_location location() { return loc; }
};

inline void log_output(log_level lev, std::string &msg, std::source_location const &loc) {
  msg = std::format("[{}] {}:{} {}", log_level_name(lev), loc.file_name(), loc.line(), msg);
  if (g_log_file)
	g_log_file << msg + '\n';

  if (lev >= g_max_level)
	std::cout << k_level_ansi_colors[(uint8_t)lev] + msg + k_reset_ansi_color + '\n';
}

template <typename ..._Args>
void generic_log(log_level lev, with_source_location<std::format_string<_Args...>> __fmt, _Args&&... __args) {
  auto const &loc = __fmt.location();
  auto msg = std::vformat(__fmt.format().get(), std::make_format_args(__args...));
  log_output(lev, msg, loc);
}

} // inner

#define _PATTERN(name) \
template <typename ..._Args> \
void log_##name(inner::with_source_location<std::format_string<_Args...>> __fmt, _Args&&... __args) { \
  inner::generic_log(log_level::name, __fmt, __args...); \
}
MINILOG_FOR_EACH(_PATTERN)
#undef _PATTERN

inline void set_log_file(std::string path) {
    inner::g_log_file = std::ofstream(path, std::ios::app);
}

inline void set_log_level(log_level lev) {
    inner::g_max_level = lev;
}

#define MINILOG_P(x) ::minilog::log_debug(#x "={}", x)

} // minilog
