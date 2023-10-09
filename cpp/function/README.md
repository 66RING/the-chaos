# function容器的实现

- abs
    * 观察到lambda的语法糖特性, 本质是自动构造一个结构体
    * 还利用了类型擦除消除了模板分别对结构体, 函数指针和lambda分别实例化
    * 类型擦除的本质是利用虚函数能提供统一接口的特性, 再自定义一个接口类将其封装起来
        + 接口类内封装base类和base类的impl类, 内含一个base类成员以获取统一接口, impl类继承自base类那么就可以使用模板自动构造类

实现可以将lambda函数函数传入容器中。举个例子:

```cpp
int main() {
  int x = 2;
  std::function<void()> f = [x]() {
	std::cout << x << std::endl;
  };

  f();
  return 0;
}
```

c语言中就可以使用函数指针, 但只可以使用全局函数。

```cpp
typedef void(*pfunc_t)();

void sayhi() {
  std::cout << "hi\n";
}

int main() {
  pfunc_t ff = sayhi;
  ff();
  return 0;
}
```

c中捕获变量引用则需要通过参数传递进去。如:

```cpp
typedef void(*pfunc_t)(void *);

struct Args {
  int &x;
  int &y;
};

void sayhi(void *args) {
  auto ax = (Args*)args;
  std::cout << ax->x << " " << ax->y << std::endl;
}

int main() {
  int x, y;
  std::cin >> x >> y;
  Args a{x, y};
  pfunc_t ff = sayhi;
  ff(&a);
  return 0;
}
```


## cpp中的function容器

> 能够捕获局部函数和局部变量

本质就是使用cpp的**语法糖**。lambda是一种快速构造的方式, 见如下代码, lambda本质就是cpp自建一个类似`func_t`的结构体, 可以使用[cppinsights](https://cppinsights.io/)查看自动构造的过程。

```cpp
struct func_t {
  void operator()() const { // 默认const, 除非lambda用mutable修饰
	std::cout << x << " " << y << std::endl;
  }
  int &x;
  int &y;
};

template <class Fn>
void calltwice(Fn func) {
  func();
  func();
}

int main() {
  int x, y;
  std::cin >> x >> y;
  func_t f{x, y};
  calltwice(f);
  // same as above
  calltwice([&x, &y] {
	  std::cout << x << " " << y << std::endl;
  });
  return 0;
}
```

如果使用模板会实例化很多遍, struct的, lambda的:

```cpp
template <class Fn>
void calltwice(Fn func) {
  func();
  func();
}
```

使用`std::function<Fn>`就只需要实例化一遍了, 这就需要用到类型擦除(擦除特化)。所以我们这里实现一个青春版: `Function<Fn>`

- 使用`()`运算符重载实现与普通函数的接口统一
- **类型擦除**
    * 我们知道虚基类的作用是提供一个统一的接口。即如果将子类赋值到基类则调用虚函数饰会自动调用对应的实现, 从而实现统一的管理
    * 类型擦除就是利用虚基类能提供统一接口的这个原理, 将虚基类的自动"分发"封装在一个类中, 该类再对外提供统一的模板和接口, 从而隐藏虚基类的调用

怎么个擦除法, 举个例子就是`Function<void(int)>`可以处理任何实现了`operator()(int)`的结构, 包括结构体, 函数指针, lambda。








