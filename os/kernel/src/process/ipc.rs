// basic ipc & ipc_call

// ipc 的两种数据传输机制, message和pipe
// 前者可以用于简单信号和握手,例如send(pid,cmd,fd)请求读fd文件,receive(pid)会得到一个pipe用于读取数据,对于接收方系统会给notify
// 发送内容被包含在至多 8 个字节中

// bool send(usize pid, usize payload),
// usize receive(usize pid)
// payload 内容自己协商

use error::Result;

pub mod error;

pub fn send(pid: usize, payload: usize) -> Result<()> {
    // block 当前进程并等待对方进程处理完消息
    todo!()
}

pub fn send_async(pid: usize, payload: usize) -> Result<()> {
    todo!()
}
