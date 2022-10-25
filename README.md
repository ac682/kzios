# ~~kzios~~ eRhino!

操作系统学习：RV64 嵌入式系统

## 设计

系统内核可执行文件组成为 `erhino-kernel` 和 board crate，后者作为可执行文件依赖内核库。board crate 会提供和板子有关的部分代码，包括内存布局，被选择的亚当程序，以及一些硬件信息（信息不多，在内核上不需要用到设备树）。

### 启动阶段

从内核第一行代码开始到用户程序被执行

#### boot stage#0

为进入裸 Rust 环境做准备，此时会挂起其他 hart，由 hart#0 进行初始化工作。

#### boot stage#1

初始化内存管理（包括内核自己的 Rust 堆管理），设定陷入模式，建立内存保护。
转移控制权到 board crate。

#### boot stage#2

board crate 准备板子的信息，传递给内核，内核利用这些信息获取硬件控制权。
此时应用程序执行环境被建立。
转移控制权给内核。

#### boot stage#3

内核做一些收尾工作，之后开始进程调度，内核通过系统调用服务进程。

## 进度

- [x] 进入 Rust 环境
- [ ] 陷入处理
  - [x] 捕获异常并输出陷入帧概览
  - [ ] 外部/软/时钟中断统筹与转发
- [ ] 系统调用
  - [ ] 调用框架
  - [ ] 具体调用实现
- [x] ~~线程(内核不支持线程)~~
- [ ] 进程
  - [ ] 进程管理
  - [ ] 调度和多核
- [ ] 内存分页
  - [x] map, ~~支持大中小页~~又不支持了，这么搞后期会很麻烦
  - [x] write
  - [ ] fill，作为系统系统调用被提供
  - [ ] unmap, 可能会有大页中取消次级页的复杂情况
- [ ] 信号
  - [ ] 进程的信号处理函数设置
  - [ ] 内核调用处理函数
  - [ ] 返回进程空间的跳板系统调用
- [ ] 进程级别系统服务设计
  - [ ] 终端输入输出服务
  - [ ] 文件系统服务
    - [ ] 虚拟文件系统
    - [ ] IPC 接口
- [ ] 内核 IPC
- [ ] 外围设备管理
- [ ] 用户可执行程序
- [ ] ...

## (将)受支持的平台

- qemu-system-riscv64: 1 core 8MB ram with MMU
- k210: 2 cores (suspend #1) 8MB ram with MMU

## 标准库

~~Porting std is a huge thing, I wont do it at the current stage.~~

仅提供 ~~`kinlib`~~`rinlib`

## 源码使用

构建系统用的 Justfile, 可执行名为 `/bin/just`

### 构建

```sh
just build
```

### 运行

需要 `qemu-system-riscv64`

```sh
just run
```

### 调试

用 gdb 调试会有字长问题，这里用`riscv64-elf-gdb`

```sh
just debug
```