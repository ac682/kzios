# ~~kzios~~ eRhino

操作系统学习：RV64 嵌入式系统

~~越来越大了，已经嵌不进去力~~

## 设计

参阅 `./notes/{doc}.md`

## 进度

- [x] 进入 Rust 环境
- [ ] 陷入处理
  - [x] 捕获异常并输出陷入帧概览
  - [ ] 外部/软/时钟中断统筹与转发
- [ ] 系统调用
  - [x] 调用框架
  - [ ] 具体调用实现
- [x] ~~线程(内核不支持线程)~~
- [ ] 进程
  - [ ] 进程模型
    - [x] 权限
  - [ ] 系统调用
    - [x] fork
    - [x] exit
    - [x] yield
    - [ ] wait
    - [ ] wait_for
  - [ ] 多核调度
    - [ ] 为不同核心指定调度器
- [ ] 线程
  - [ ] 线程模型
- [ ] 内存分页,~~支持大中小页~~又不支持了，这么搞后期会很麻烦
  - [x] map, 作为系统调用 sys_map 提供给具有 ProcessPermission::Memory 权限的进程
  - [x] write
  - [x] fill，作为系统系统调用 sys_extend 被提供
  - [ ] unmap, ~~可能会有大页中取消次级页的复杂情况~~
  - [ ] 多种内存分页模型
    - [ ] Sv32
    - [x] Sv39
    - [ ] Sv48
  - [x] 写时复制
- [x] 信号
  - [x] 进程的信号处理函数设置
  - [x] 内核调用处理函数
  - [x] 返回进程空间的跳板系统调用
- [ ] 进程级别系统服务设计
  - [ ] 终端输入输出服务
  - [ ] 文件系统服务
    - [ ] 虚拟文件系统
    - [ ] IPC 接口
- [ ] 内核 IPC
- [ ] SMP 支持
  - [ ] IPI
- [ ] 外围设备管理
- [x] 用户可执行程序
- [ ] ...

## (将)受支持的平台

- [x] qemu-system-riscv64: 4 cores 128MB ram with MMU
- [ ] k210: 2 cores (suspend #1) 8MB ram with MMU *内存太少了哇*
- [ ] D1s(F133): single core minimal 64mb with MMU *板子还没准备好，还没测*

## 标准库

~~Porting std is huge work, I wont do it at the current stage.~~

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
