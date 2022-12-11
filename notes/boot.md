# 启动

系统内核可执行文件组成为 `erhino-kernel` 和 board crate，后者作为可执行文件依赖内核库。board crate 会提供和板子有关的部分代码，包括内存布局，被选择的亚当程序，以及一些硬件信息（信息不多，在内核上不需要用到设备树）。

系统不提供操作系统级别线程，用户态有需求就需要自己实现线程。唯一的线程概念是位于内核的硬件线程(hart)。硬件线程作为系统资源被内核管理。

## 启动阶段

从内核第一行代码开始到用户程序被执行

### boot stage #0: _start@assembly.asm

为进入裸 Rust 环境做准备，此时会挂起其他 hart，由 hart#0 进行初始化工作。

### boot stage #1: rust_start@rt.rs

初始化内核自己的 Rust 堆管理
转移控制权到 board crate。

### boot stage #2: main@board_crate->kernel_init@lib,rs->kernel_main@lib,rs

board crate 准备板子的信息，传递给内核，内核利用这些信息获取硬件控制权。
设定陷入模式，建立内存保护，开启内核各种服务。此时应用程序执行环境被建立。
转移控制权给内核。

### boot stage #3: in kernel call

内核做一些收尾工作，之后开始进程调度，内核通过系统调用服务进程。
