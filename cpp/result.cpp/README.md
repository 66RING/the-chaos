# Result.cpp

> implement rust result in cpp.
>
> 本质就是一个`Result<T, U>`类使用union记录有两个成员ok/err

- Usage/API
    * `result::Result<int, std::string>::ok(1)`
    * `result::Result<int, std::string>::error("fail")`
    * `unwrap`, `unwrap_or(value)`, ...
    * `is_ok()`, `is_err()`

首先需要两个类型`Result<Value, Error>`, 然后提供`ok()`和`error()`构造。使用`union`保存内部值, 而构造时`union`时可以使用传入一个空参数作为hint优雅的构造, 如`Result<Value, Error>(type::ok, internal::storage_ok, std::forward<T>(value)...)`, `internal::storage_ok`没有形参, 只用来触发不同的构造方法。

## impl

- 使用union存储值, 使用enum表示类型

```cpp
class Result {
    // 值存储
    union storage {
        Value ok;
        Error err;
    };
    // 类型标注
    enum class type: unsigned char {
        ok,
        error
    };
}
```

- 使用`is_nothrow_constructible<T>::value`判断类型
- 删除默认构造函数`Result() = delete`, 使用`ok()`, `error()`等做构造: 返回Result对象

```cpp
class Result {
    private:
        // 类型标注
        enum class type: unsigned char {
            ok,
            error
        };
        type variant;
        // 值存储
        storage store;

        explicit Result(type variant, A&&... value) noexcept(is_constructor_noexcept) : variant(variant), store(std::forward<A>(value)...) {}
    public:
        ///Creates Ok variant.
        template<class... T>
        static Result<Value, Error> ok(T&&... value) noexcept(storage::is_value_noexcept) {
            return Result<Value, Error>(type::ok, internal::storage_ok, std::forward<T>(value)...);
        }

        ///Creates Error variant.
        template<class... E>
        static Result<Value, Error> error(E&&... error) noexcept(storage::is_error_noexcept) {
            return Result<Value, Error>(type::error, internal::storage_error, std::forward<E>(error)...);
        }
}
```

- 完善可用性:
    * 实现Result, ok, err的移动构造, `&T`构造等
    * 重载`=`运算符
    * API
        + `is_ok`, `is_err()`
        + `operator bool()`: 到bool的隐式转换
        + `Value* value()`返回ok的值或者nullptr
        + `Error* error()`返回error的值或者nullptr
        + `unwrap`, 返回ok或throw错误, **都是返回内部的引用**
        + `map(fn)`取出内部值, error则直接返回, ok则通过fn处理后再返回
        + `and_then(fn)`, 类似map都会移出原来的值, 但是可以不返回Result, 可以返回新值

为什么Result的泛型不作为直接作为内部的泛型? 外部`Value`目标参数的用来标注类型的, 内部`T`是用来传递可变参数的。

```cpp
template<class Value, class Error>
class Result {
    public:
        ///Creates Ok variant.
        template<class... T>
        static Result<Value, Error> ok(T&&... value) noexcept(storage::is_value_noexcept) {
            return Result<Value, Error>(type::ok, internal::storage_ok, std::forward<T>(value)...);
        }

};
```

map的实现: 返回一个新的Result, Result中ok的类型可能改变, 取决于Fn的返回值。

```cpp
template<typename Fn, typename NewValue = std::invoke_result_t<Fn, Value>>
constexpr Result<NewValue, Error> map(Fn fn) {
    static_assert(std::is_invocable<Fn, Value>::value, "Fn must be callable and accept Value as argument");

    if (is_err()) {
        Error error = std::move(this->store.error);
        return Result<NewValue, Error>::error(error);
    } else {
        Value value = std::move(this->store.ok);
        return Result<NewValue, Error>::ok(fn(value));
    }
}
```


## cpp misc

- `operator bool()`: 表示类到bool的隐式转换

- `std::is_trivially_destructible<T>::value`
- `std::is_nothrow_move_constructible<T>::value`

cpp throw的用法, 如果不做处理自动终止程序。

```cpp
///Attempts to unwrap result, yielding content of Ok.
///
///@throws Content of Error.
constexpr Value& unwrap() & {
    if (is_ok()) {
        return store.ok;
    } else {
        throw store.error;
    }
}
```

后面还有`const & noexcept`是什么语法: 

1. `const &`表示的是调用者的类型, 是一种重载
2. `noexcept`表示不会抛出异常, 让编译器更激进优化

```cpp
///Attempts to unwrap result, yielding content of Ok or, if it is not ok, other.
constexpr Value unwrap_or(Value&& other) const & noexcept(std::is_nothrow_move_constructible<Value>::value && std::is_nothrow_copy_constructible<Value>::value) {
    return is_ok() ? store.ok : std::move(other);
}
```

### enum class: T

旧enum存在许多问题:

1. 隐式转换成整型
2. 无法自定义类型
3. 存在作用域问题, 可以直接通过enum的成员名访问成员
4. 取决于编译

`enum class Name {}`语法解决了旧`enum`的问题

1. 不再隐式转换, 可以手动强转
2. 指定底层数据类型: `enum class Name: T {}`
3. 作用域访问成员需要使用域运算符


### noexcept的功能

- 方便编译器优化
- 有条件的noexcept: `noexcept(expr)`


### other

- 函数后置const, 表示常成员函数, 不能修改成员的值
- 成员函数后加`&`和`&&`表示类对象(this)的类型, 用于重载
- `is_invocable<Fn, Arg_T>`, 检查函数参数匹配

