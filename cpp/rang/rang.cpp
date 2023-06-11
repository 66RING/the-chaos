#include <iostream>
#include <cstring>

using namespace std;

namespace rang {

enum class style {
    reset     = 0,
    bold      = 1,
    dim       = 2,
    italic    = 3,
    underline = 4,
    blink     = 5,
    rblink    = 6,
    reversed  = 7,
    conceal   = 8,
    crossed   = 9
};

// 详情请看ASCII color code: https://en.wikipedia.org/wiki/ANSI_escape_code
enum class fg {
    black   = 30,
    red     = 31,
    green   = 32,
    yellow  = 33,
    blue    = 34,
    magenta = 35,
    cyan    = 36,
    gray    = 37,
    reset   = 39
};

enum class bg {
    black   = 40,
    red     = 41,
    green   = 42,
    yellow  = 43,
    blue    = 44,
    magenta = 45,
    cyan    = 46,
    gray    = 47,
    reset   = 49
};

enum class fgB {
    black   = 90,
    red     = 91,
    green   = 92,
    yellow  = 93,
    blue    = 94,
    magenta = 95,
    cyan    = 96,
    gray    = 97
};

enum class bgB {
    black   = 100,
    red     = 101,
    green   = 102,
    yellow  = 103,
    blue    = 104,
    magenta = 105,
    cyan    = 106,
    gray    = 107
};


template <typename T>
using enableStd = typename std::enable_if<
  std::is_same<T, rang::style>::value || std::is_same<T, rang::fg>::value
	|| std::is_same<T, rang::bg>::value,
  std::ostream &>::type;

// 设置颜色
template <typename T>
inline enableStd<T> setColor(std::ostream &os, T const value) {
  return os << "\033[" << static_cast<int>(value) << "m";
}

// 重载<<运算符以方便使用
template <typename T>
inline enableStd<T> operator<<(std::ostream &os, T const value) {
  return setColor(os, value);
}

}

int main()
{
  using namespace rang;

  cout << style::reset << style::bold << fg::green << bg::red << "green fg, reb bg\n" 
	<< style::reset << style::underline << fg::blue << bg::yellow << "blue fg, reb bg\n" 
	<< style::reset << style::blink << fg::gray << bg::magenta << "gray fg, magenta bg" << style::reset << endl;
}
