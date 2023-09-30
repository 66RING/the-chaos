# cpp client

一般一个client只面向一个server, 所以这里client的设计就可以简单粗暴一些: 直接read write。所以这里还是面向过程的思想:

> client可以有多种版本, 只要遵从传输协议就可以使用

1. init sock/tcp, get fd
2. malloc read, write buffer
3. read from stdin
4. send(fd)
5. while not done { read(fd) }

