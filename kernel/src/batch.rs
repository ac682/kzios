use lazy_static::*;

use crate::sync::safe_cell::SafeCell;

const MAX_APP_NUM: usize = 32;

pub fn init(){
    todo!();
}

pub struct AppManager{
    num_app: usize,
    current_app: usize,
    app_start: [usize; MAX_APP_NUM + 1]
}

lazy_static!{
    pub static ref APP_MANAGER: SafeCell<AppManager> = unsafe{
        SafeCell::new({
            extern "C"{
                fn _num_app();
            }
            let num_app_ptr = _num_app as usize as *const usize;
            let num_app = num_app_ptr.read_volatile();
            let mut app_start: [usize; MAX_APP_NUM + 1] = [0; MAX_APP_NUM + 1];
            let app_start_raw: &[usize] =
                core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1);
            app_start[..=num_app].copy_from_slice(app_start_raw);
            AppManager {
                num_app,
                current_app: 0,
                app_start,
            }
        })
    };
}