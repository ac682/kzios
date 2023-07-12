# eRhino

操作系统学习：RV64

## 设计

参阅 `./notes/{doc}.md`

## 进度

没有进度了，开始重构了

## (将)受支持的平台

- [x] qemu-system-riscv64: 4 cores 128MB ram with MMU
- [ ] k210: 2 cores 8MB ram with MMU *内存太少了哇*
- [ ] D1s(F133): single core minimal 64mb with MMU

在重构，以上连模拟器都还没测

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
