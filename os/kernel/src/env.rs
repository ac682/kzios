#[export_name = "_env"]
static ENVIRONMENT: Environment = Environment { boot_args: [0; 8] };

#[repr(C)]
pub struct Environment {
    boot_args: [u64; 8],
}

pub fn args() -> &'static [u64; 8] {
    &ENVIRONMENT.boot_args
}
