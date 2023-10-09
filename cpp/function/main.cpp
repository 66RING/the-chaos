#include <functional>
#include <iostream>
#include <stdexcept>
#include <type_traits>
#include <utility>
#include <vector>
#include <memory>

// NOTE: 关闭默认特化, 必须使用函数形式特化
// 为什么不能直接false, 因为要依赖模板FnSig做惰性编译
template <class FnSig>
struct Function {
  static_assert(!std::is_same_v<FnSig, FnSig>, "not a valid function signature");
};

template <class Ret, class ...Args>
// 特化: 使用时用这种格式, Function<void(int)>
// NOTE: 偏特化: 如果存在这种形式就不会跳到上面FnSig处了
struct Function<Ret(Args...)> {
  struct FuncBase {
	virtual Ret call(Args ...args) = 0;
	virtual ~FuncBase() = default;
  };

  template <class F>
  struct FuncImpl : FuncBase {
	F f;

	FuncImpl(F f) : f(std::move(f)) {}

	// 把FuncBase的call覆盖掉, 就是调用传入的函数F
	virtual Ret call(Args ...args) override {
	  return std::invoke(f, std::forward<Args>(args)...);
	}
  };

  std::shared_ptr<FuncBase> m_base;

  Function() = default;

  // 构造函数, 会实例化多次, 但都用统一的虚接口
  template <class F, class = std::enable_if_t<std::is_invocable_r_v<Ret, F &, Args...>>>
  Function(F f): m_base(std::make_shared<FuncImpl<F>>(std::move(f))) {}

  Ret operator()(Args ...args) const {
	if (!m_base) [[unlikely]]
	  throw std::runtime_error("function uninitialized");
	// 虚接口实例化多次, 虚函数自动调用对应实现
	return m_base->call(std::forward<Args>(args)...);
  }
};

void func_hello(int i) {
  std::cout << "hello\n";
}

// ⭐任何提供operator()的都能包起来
struct func_printnum_t {
  void operator()(int i) {
	std::cout << i << " " << x << " " << y << std::endl;
  }
  int x, y;
};

// 如果
// 不需要模板参数, 从而分离声明和定义
// 这里用到了类型擦除
// 等价于std::function<void(int)>
void calltwice(Function<void(int)> const &func) {
  func(1);
  func(2);
}

int main() {
  int x, y;
  std::cin >> x >> y;
  // calltwice([=](int i) {
	  // std::cout << i << " " << x << " " << y << std::endl;
  // });
  func_printnum_t f(x, y);
  calltwice(f);

  return 0;
}
