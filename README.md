# eRhino

操作系统学习：RV64

## 设计

参阅 `./notes/{doc}.md`

## 进度

- [ ] IPC
  - [x] 信号
  - [ ] 消息
  - [ ] 隧道
    - [x] syscall
    - [ ] Runnel
- [ ] 设备租借
  - [ ] 中断转发
- [ ] 文件系统
  - [ ] FAL/syscall
    - [x] access
    - [x] inspect
    - [x] read
    - [ ] write
    - [ ] create
    - [ ] delete
    - [ ] open
  - [ ] FAL/ipc
  - [ ] 内核文件系统
    - [ ] rootfs
    - [ ] procfs
    - [ ] devfs
  - [ ] 具体文件系统
    - [ ] FAT32

## (将)受支持的平台

- [x] qemu-virt: 4 cores 128MB ram with MMU
- [x] qemu-sifive_u: 5 cores(#0 disabled) 128MB ram with MMU
- [ ] k210: 2 cores 8MB ram with MMU *内存太少了哇*
- [ ] D1s(F133): single core 64MB with MMU

只有 virt 能跑，其他的会遇到莫名bug

## 标准库

~~Porting std is huge work, I wont do it at the current stage.~~

仅提供 `rinlib`

## 源码使用

构建系统用的 Justfile, 可执行名为 `/bin/just`

使用参数 `PLATFORM` 和 `MODEL` 来对应 `platforms/{{PLATFORM}}/{{MODEL}}/*` 的设备文件。
由于我不知道如何用 just 做到使用 dict 保存并应用 PLATFORM/MODEL 到 OpenSBI/PLATFORM/FW_CONFIG 的特定配置，编译或者运行需要 `just build(或run)_$MODEL`，要求指定型号。

### 构建

```sh
just PLATFORM=foo MODEL=bar build
```

### 运行

需要 `qemu-system-riscv64`

```sh
just run
```

### 调试

~~~用 gdb 调试会有字长问题，这里用`riscv64-elf-gdb`~~~
用 riscv64-elf-gdb 调试会有 rust-testsuit 问题，这里用 `git-multiarch`

```sh
just debug
```
