// 扁平化的设备描述，能够被一个线性容器存储
// 两种方案，一种是 Rust 静态的 tagged union 来表示所有可能的设备类型，并提供对应的结构体来储存必要的信息，来表示该设备被内核识别且支持，接口可以针对设备制定
// 而是直接照搬设备树内容
// 因为驱动都在用户空间，内核不关心设备类型，也不会提供针对特定设备的接口，所以实际不需要识别类型，只需要做好特性分类即可
//
// 方案：
// Vec<GenericDevice>, 因为这个列表是不可变的， 使用 index 来表示其唯一 id, 包括用户态。
// 设备属性被储存在 traits 中
// 除此还有常规属性，type(node.type_name), name(node.name) 等和引用的中断控制器
// type 仅供参考，能力还是得从 traits 中提取
//
// 关于引用，引用只使用 id，使用 parent 来表示从属关系，依赖例如 clocks = <&hclk, &pclk>
// 那么就要求 hclk 和 pclk 已经被加入到列表中
// 如果没遍历到 hclk，那么这里就使用另一种引用法，Handle(&hclk)，在构建列表的时候再去不断迭代，不断减少 Handle 引用直到全部消失
// 这里使用 GenericDeviceBuilder 来保存虚空引用
// Handle 还要考虑到 parent 分配，由于 PHandle 是 32 位的，所以 parent 直接从 1<<32 开始计数
//
// Trait:
// Interrupt-Driven()

use alloc::string::String;

pub struct GenericDevice{
    // /sys/dev/{id}/compatible or not exist
    name: String,
    typ: String,
    compatible: Option<String>,
    parent: Option<usize>
}

pub struct GenericDeviceBuilder{
    handle: usize,
    parent: Option<usize>
}