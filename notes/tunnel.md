# 跨空间内存交换

提供一种基于共享内存页的内核/进程，进程/进程，进程/内核的匿名数据交换方式。支持单双工，由协议决定。
由于是匿名的，通过系统调用 Open 创建的通道无法确定是哪个文件系统服务进程提供的。

## 实现

在双方的内存空间中映射同一个页帧，使用特定交换协议实现对数据交换。
内核提供类似中断的方式来减少盲等数据，使用系统调用发送，对于进程会转发到信号，对于内核会直接处理拦截该系统调用。

如何使用该中断由协议决定，特定协议可以不支持中断，也可以由数据交换双方约定如何使用中断，一种可能的实现可以参考下面。

### Runnel，一种通用的用于流式数据的交换协议(FIFO，单工)

Halcyon 全面支持该协议并用于 fal。细节如下。

在共享页面中储存一个页大小的数据结构，包含缓冲区和控制块。缓冲区分成 1k 大小的块，最大三个，整个页剩下的 1k 留给控制块。控制块中记录缓冲区数据有效信息，发送端信息，接收端信息。
发送端在写入数据时需要先判断缓冲区数据可用性，结合控制块中对应信息判断是否可写，接收端同理。

数据传送触发除了盲等，还支持主动请求数据(是否启用中断)，和超时请求。以向进程请求文件为例，A 发送携带文件信息的消息请求 B，B 同意并创建隧道，发送回执，回执中会包含触发方式。

### 缺陷

由于没有内核参与，接收端无法判断是否有数据到来，不得不线程不断监听检查，发送端也会遇到接收端睡死无法读取导致缓冲区一直不可用而被迫死锁。

## 安全

不安全，仅需要内存拷贝，不用系统调用，快！