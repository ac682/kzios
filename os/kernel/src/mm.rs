use erhino_shared::mem::Address;

pub mod frame;
pub mod page;
pub mod range;
pub mod unit;

pub fn physical_copy(_from: Address, _to: Address, _length: usize) {
    // 按usize大小去拷贝
    todo!()
}
