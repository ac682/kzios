# 内核调用

分为两种，Remote Call 和 System Cal。

## Remote Call

远程调用。
由 `ApplicationHart` 发送给另一个同类型 hart，没有返回值，用 IPI 实现。目前没实现也没应用。

## System Call

系统调用。
由进程调用内核，用 Ecall 实现，有返回值。有两种调用类型，异步和同步。同步调用是最常用的类型，也是大部分系统调用的实现，异步调用不会直接返回结果，而是先调度掉进程，等任务完成之后再返回结果。
