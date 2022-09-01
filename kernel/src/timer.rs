use riscv::register::mie;

pub fn init(){
    unsafe{
        mie::set_mtimer();
        mie::set_stimer();
        mie::set_utimer();
    }
}

pub fn set_next_timer() {
    unsafe {
        let mtimecmp = 0x0200_4000 as *mut u64;
        let mtime = 0x0200_bff8 as *const u64;
        // The frequency given by QEMU is 10_000_000 Hz, so this sets
        // the next interrupt to fire one second from now.
        // This is much too slow for normal operations, but it gives us
        // a visual of what's happening behind the scenes.
        // 10ms
        mtimecmp.write_volatile(mtime.read_volatile() + 10_000_000);
    }
}
