#include <type_traits>
#include <utility>

#include<iostream>
#include<vector>

namespace _hint {
  // 通过函数重载, 在参数列表中作为构造函数的标识
  struct storage_ok_t { constexpr storage_ok_t() {} };
  struct storage_err_t { constexpr storage_err_t() {} };
  struct storage_empty_t { constexpr storage_empty_t() {} };

  // 用于标识的变量
  constexpr storage_ok_t storage_ok;
  constexpr storage_err_t storage_err;
  constexpr storage_empty_t storage_empty;
}

template<class Value>
class Ok {
public:
  Value inner;
  Ok() = delete;
  Ok(const Value& v): inner(v) {};
  Ok(Value&& v): inner(std::move(v)) {};
};

template<class Value>
class Err {
public:
  Value inner;
  Err() = delete;
  Err(const Value& v): inner(v) {};
  Err(Value&& v): inner(std::move(v)) {};
};


template<class Value, class Error>
class Result {
private:
  // union存储内部值
  union storage {
	Value ok;
	Error err;
	// 提供不同函数前面以对外提供调用
	template<typename ...T>
	explicit storage(_hint::storage_ok_t, T&&... a): ok(std::forward<T>(a)...) {}

	template<typename ...T>
	explicit storage(_hint::storage_err_t, T&&... a): err(std::forward<T>(a)...) {}

	template<typename ...T>
	explicit storage(_hint::storage_empty_t, T&&... a) {}

	~storage() {}
  };
  // enum存储类型标识
  enum class type: unsigned char {
	ok,
	error,
  };
  storage store_;
  type variant_;

  template<class... T>
  Result(type variant, T&&... args): variant_(variant), store_(std::forward<T>(args)...) {}


public:
  template<class... T>
  static Result<Value, Error> ok(T&&... value) {
	return Result<Value, Error>(type::ok, _hint::storage_ok, std::forward<T>(value)...);
  }

  template<class... T>
  static Result<Value, Error> error(T&&... error) {
	return Result<Value, Error>(type::error, _hint::storage_err, std::forward<T>(error)...);
  }

  constexpr Value& unwrap() & {
	if (is_ok()) {
	  return store_.ok;
	} else {
	  throw store_.err;
	}
  }

  constexpr const Value& unwrap() const & {
	const_cast<Result*>(this)->unwrap();
  }

  constexpr Value unwrap() && {
	if (is_ok()) {
	  return std::move(store_.ok);
	} else {
	  throw store_.err;
	}
  }

  template<typename Fn, typename NewValue = std::invoke_result_t<Fn, Value>>
  constexpr Result<NewValue, Error> map(Fn fn) {
	static_assert(std::is_invocable<Fn, Value>::value, "Fn must be callable and accept Value as argument");

	if (is_error()) {
	  Error err = std::move(store_.err);
	  return Result<NewValue, Error>::error(err);
	} else {
	  Value value = std::move(store_.ok);
	  return Result<NewValue, Error>::ok(fn(value));
	}
  }

  constexpr bool is_ok() const noexcept {
	return variant_ == type::ok;
  }
  constexpr bool is_error() const noexcept {
	return variant_ == type::error;
  }
};

int main() {
  using namespace std;

  {
	auto ok = Result<int, int>::ok(100);
	auto err = Result<int, int>::error(233);
	cout << ok.is_ok() << endl;
	cout << ok.is_error() << endl;
	cout << ok.unwrap() << endl;
	// cout << err.unwrap() << endl;

	auto l = [](int) {
	  return 1;
	};

	auto x = ok.map(l);
	cout << x.unwrap() << endl;
	cout << ok.unwrap() << endl;
  }

  return 0;
}

