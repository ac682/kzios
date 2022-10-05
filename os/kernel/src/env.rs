#[export_name = "_env"]
static ENVIRONMENT: Environment = Environment{
    boot_args:[0;7]
};

#[repr(C)]
pub struct Environment{
    boot_args: [u64; 7],
}

pub fn args() -> &'static [u64; 7]{
    &ENVIRONMENT.boot_args
}