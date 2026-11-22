use anchor_lang::{Address, event};

#[event(bytemuck)]
#[repr(C)]
pub struct Deposited {
    pub depositor: Address,
    pub mint: Address,
    pub amount: u64,
}
