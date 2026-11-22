use anchor_lang::{Address, event};

#[event(bytemuck)]
#[repr(C)]
pub struct Withdrawn {
    pub authority: Address,
    pub mint: Address,
    pub amount: u64,
}
