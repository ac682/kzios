// Services
// |- Hardware
// |--|- spi0
// |--|--|- Interface(String Property): spi
// |--|--|- Sid(Integer Property): 1
// |--|--|- Seat(Integer Property, Pid): 114
// |--|- mmc_spi0
// |--|--|- Interface(Integer Property): mmc_spi
// |--|--|- Sid(Integer Property): 2
// |--|--|- Host(Integer Property, Sid): 1

// TODO: ServiceNode 的一维有序数组，按 sid 升序，加锁，采用二分查找。
// Register 会创建一个 Node，但没有 seat，Claim 则会像对应 sid 的 Node 填入 pid 到 seat。
// 内核会把 dt 转移到 /Services/Hardware 中，由 init 进程负责检索并拉起驱动，被拉起的驱动需要对号入座。
// 上述的文件系统目录由 srvfs 创建。
// TODO: srvfs 需要一种方法来监听创建和移除。

use alloc::string::String;
use erhino_shared::proc::Pid;

type Sid = usize;

pub enum ServiceType {}

pub struct ServiceNode {
    sid: Sid,
    typ: ServiceType,
    interface: String,
    seat: Option<Pid>,
    host: Option<Sid>,
}
