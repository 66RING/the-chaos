#include <iostream>
#include <assert.h>
#include <mutex>
#include <thread>
#include <vector>

using namespace std;

class test {
public:
  test(string name, int age): name_(name), age_(age) {};
  string name_;
  int age_;
};

template <typename T> 
class smart_ptr {
public:
  // 指针构造
  smart_ptr(T* ptr = nullptr) {
	ptr_ = ptr;
	mutex_ = new mutex();
	if (ptr == nullptr) {
	  ref_count_ = new size_t(0);
	} else {
	  ref_count_ = new size_t(1);
	}
  }

  ~smart_ptr() {
	if (ptr_ == nullptr) {
	  cout << "clear nullptr\n";
	  delete ref_count_;
	  delete mutex_;
	  return;
	}

	cout << "~ ptr\n";

	// 没有共享就释放底层数据
	rc_down();
  }

  // 拷贝构造
  smart_ptr(smart_ptr& sp) {
	// 空智能指针则直接再初始一个空智能指针
	if (sp.ptr_ == nullptr) {
	  ptr_ = nullptr;
	  ref_count_ = new size_t(0);
	  mutex_ = new mutex();
	  return;
	}

	// 同一指针无需拷贝
	if (sp.ptr_ == ptr_)
	  return;

	// 更新新指针
	ptr_ = sp.ptr_;
	ref_count_ = sp.ref_count_;
	mutex_ = sp.mutex_;
	rc_up();
  }

  // 重载=运算符
  smart_ptr& operator=(smart_ptr& sp) {
	// 空智能指针则直接再初始一个空智能指针
	if (sp->ptr_ == nullptr) {
	  ptr_ = nullptr;
	  ref_count_ = new size_t(0);
	  mutex_ = new mutex();
	  return *this;
	}

	// 同一指针无需拷贝
	if (sp->ptr_ == ptr_)
	  return *this;

	// 解决旧指针
	rc_down();

	// 更新新指针
	ptr_ = sp->ptr_;
	ref_count_ = sp->ref_count_;
	mutex_ = sp.mutex_;
	rc_up();

	return *this;
  }

  // 重载->运算符
  T* operator->() {
	assert(ptr_ != nullptr);
	return ptr_;
  }

  // 重载*解引用运算符
  T& operator*() {
	assert(ptr_ != nullptr);
	return *ptr_;
  }

  // 重载解引用运算符

  size_t ref_count() {
	return *ref_count_;
  }
private:
  void rc_down() {
	bool should_free = false;
	mutex_->lock();
	(*ref_count_)--;
	if (*ref_count_ == 0) {
	  cout << "clear ptr\n";
	  should_free = true;
	  delete ref_count_;
	  delete ptr_;
	}
	mutex_->unlock();
	if (should_free)
	  delete mutex_;
  }

  void rc_up() {
	mutex_->lock();
	(*ref_count_)++;
	mutex_->unlock();
  }

  T* ptr_;
  size_t* ref_count_;
  mutex* mutex_;
};

int main() { 
  // 基础功能测试
  {
	// 测试指针构造
	smart_ptr<test> sp(new test("hi", 10));

	// 测试运算符重载
	assert(sp->name_ == "hi");
	assert(sp->age_ == 10);
	assert(sp.ref_count() == 1);

	// 测试空指针构造
	{
	  smart_ptr<test> sp2;
	  assert(sp2.ref_count() == 0);
	  // 测试空指针删除
	}

	// 测试拷贝构造
	smart_ptr<test> sp3(sp);
	assert(sp3->name_ == "hi");
	assert(sp3->age_ == 10);
	assert(sp3.ref_count() == 2);

	// 测试=运算符
	smart_ptr<test> sp4 = sp;
	assert(sp4->name_ == "hi");
	assert(sp4->age_ == 10);
	assert(sp4.ref_count() == 3);

	// 测试*运算符
	(*sp4).name_ = "hiii";
	(*sp4).age_ = 100;
	assert(sp->name_ == "hiii");
	assert(sp->age_ == 100);
	assert(sp3->name_ == "hiii");
	assert(sp3->age_ == 100);
	assert(sp4->name_ == "hiii");
	assert(sp4->age_ == 100);
  }

  // 测试空指针拷贝
  {
	smart_ptr<test> nsp1;
	smart_ptr<test> nsp2 = nsp1;
	smart_ptr<test> nsp3(nsp1);
	assert(nsp1.ref_count() == 0);
	assert(nsp2.ref_count() == 0);
	assert(nsp3.ref_count() == 0);
  }

  // 测试并发
  {
	cout << "multi thread test start." << endl;
	vector<thread> ts;
	smart_ptr<test> mtsp(new test("ts", 0));
	for (int i = 0; i < 100; i++) {
	  ts.emplace_back([&]() {
	  smart_ptr<test> mtsp1(mtsp);
	  smart_ptr<test> mtsp2 = mtsp;
	  });
	}
	for (auto &t: ts) {
	  if (t.joinable())
		t.join();
	}

	assert(mtsp.ref_count() == 1);
	assert(mtsp->name_ == "ts");
	assert(mtsp->age_ == 0);
	cout << "multi thread test done." << endl;
  }

  
  return 0; 
}
