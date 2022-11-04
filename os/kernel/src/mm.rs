use erhino_shared::mem::Address;

pub mod frame;
pub mod page;
pub mod range;
pub mod unit;

pub fn physical_copy(from: Address, to: Address,length: usize){
    // 按usize大小去拷贝
    todo!()
}