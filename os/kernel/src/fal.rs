use alloc::boxed::Box;
use spin::Once;

static MASTER: Once<Register> = Once::new();

pub struct Register{
    
}

pub enum Node{
    Directory,
    File,
    MountPoint(Box<Register>)
}