#include <condition_variable>
#include <functional>
#include <iostream>
#include <mutex>
#include <queue>
#include <thread>
#include <vector>

using namespace std;

class ThreadPool {
public:
  /*
   * 创建线程池
   * @param: 创建num_thread个worker线程
   */
  ThreadPool(int num_thread) {
    for (int i = 0; i < num_thread; i++) {
      // 添加worker线程
      AddThread();
    }
  }

  ~ThreadPool() {
    {
      unique_lock<mutex> lock(mutex);
      // 停止标志位
      stop_ = true;
    }

    // 唤醒所有线程
    cv_.notify_all();

    // NOTE:细节引用
    for (auto &thread : pool_) {
      // 等待每个线程结束
      if (thread.joinable())
        thread.join();
    }
  }

  /*
   * 添加线程
   */
  void AddThread() {
	// 启动若干worker线程
    pool_.emplace_back([this]() {
      // 添加worker线程
      // 不断执行任务队列中的任务
      while (true) {
        function<void()> task;
        {
		  // 1. 这个锁的用法
		  // 	- 通过引用传递给cv, 使得cv模块可以控制外部行为
		  // 2. 搞清楚这个锁wait后不会死锁吗?
		  // 	- 让出线程时会释放锁, 恢复时又自动上锁
          unique_lock<mutex> lock(mutex_);

          // 等待任务到来或停止
          cv_.wait(lock, [this]() { return !tasks_.empty() || stop_; });

          if (stop_ && tasks_.empty()) {
            // 停止线程
            return;
          }

		  // 取出任务
		  task = std::move(tasks_.front());
		  tasks_.pop();
        }

		// 执行任务
		task();
      }
    });
  }

  // 提交任务到任务队列
  // 完美转发 + 可变参数模板
  template<typename F, typename... Args>
  void Commit(F &&f, Args &&...args) {
	// bind构建可调用对象
	auto task = bind(std::forward<F>(f), std::forward<Args>(args)...);

	{
	  unique_lock<mutex> lock(mutex_);
	  tasks_.emplace(task);
	}
	// 通知一个正在等待的线程
	cv_.notify_one();
  }

private:
  mutex mutex_;
  // 停止标志位
  bool stop_ {false};
  condition_variable cv_;
  // 线程池
  vector<thread> pool_;
  // 任务队列
  queue<function<void()>> tasks_;
};

void PrintTask(int num) {
  cout << "Thread ID: " << this_thread::get_id() << " -> " << num << endl;
}

int main() {
  ThreadPool pool(4);

  // 添加任务到线程池
  for (int i = 0; i < 10; i++) {
	pool.Commit(PrintTask, i);
  }

  return 0;
}


