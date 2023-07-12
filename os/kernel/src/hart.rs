use crate::trap::TrapFrame;

const HART_NUM_MAX: u8 = u8::MAX;

static mut HARTS: [Option<Hart>; HART_NUM_MAX as usize] = [None; HART_NUM_MAX as usize];

#[derive(Clone, Copy)]
pub struct Hart {
    id: u8,
    trash_bin: TrapFrame,
}

impl Hart {
    pub const fn new(hartid: u8) -> Self {
        Self {
            id: hartid,
            trash_bin: TrapFrame::new(hartid),
        }
    }

    pub fn init() {
        // call on boot
    }
}
