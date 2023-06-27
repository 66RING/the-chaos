# 写System daemon程序

> 本质就是linux系统的各种系统调用, 走完整的daemon创建流程, 开干净的session, 用户组等。面面俱到的设置

- abs
    * **setid, dup2, chroot, getpwnam, getgrnam**
    * setsid, umask, chown
    * fork, open


## 使用/接口

```rust
extern crate daemonize;

use std::fs::File;

use daemonize::Daemonize;

fn main() {
    let stdout = File::create("/tmp/daemon.out").unwrap();
    let stderr = File::create("/tmp/daemon.err").unwrap();

    let daemonize = Daemonize::new()
        .pid_file("/tmp/test.pid") // Every method except `new` and `start`
        .chown_pid_file(true)      // is optional, see `Daemonize` documentation
        .working_directory("/tmp") // for default behaviour.
        .user("nobody")
        .group("daemon") // Group name
        .group(2)        // or group id.
        .umask(0o777)    // Set umask, `0o027` by default.
        .stdout(stdout)  // Redirect stdout to `/tmp/daemon.out`.
        .stderr(stderr)  // Redirect stderr to `/tmp/daemon.err`.
        .privileged_action(|| "Executed before drop privileges");

    match daemonize.start() {
        Ok(_) => println!("Success, daemonized"),
        Err(e) => eprintln!("Error, {}", e),
    }
}
```

## 实现

结构: 记录所有信息即可

```rust
pub struct Daemonize<T> {
    // PathBuf, 跨平台文件路径名
    directory: PathBuf,
    pid_file: Option<PathBuf>,
    chown_pid_file: bool,
    user: Option<User>,
    group: Option<Group>,
    umask: Mask,
    root: Option<PathBuf>,
    privileged_action: Box<dyn FnOnce() -> T>,
    stdin: Stdio,
    stdout: Stdio,
    stderr: Stdio,
}
```

- 各种赋值成员的helper函数
    - 略
- execute
    * fork
        + 父进程waitpid等待子进程结束
        + 子进程**`execute_child`**, just ask man
            + 切换工作目录
            + 启动新的sessino: setsid
            + umask
            + 然后再fork出真正的目标进程, 这里的父进程直接退出
            + 创建指定创建的文件, `libc::open`
            + 处理重定向: 单独处理stdin, stdout, stderr
                + dup2系统调用
            + 获取指定用户信息然后chown
                + getpwnam, 通过string获取uid
                + getgrnam, 通过string获取gid
                + chown
            + 切换shell根目录: chroot
- start
    * 调用`execute`



