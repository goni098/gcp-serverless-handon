use anchor_lang::{Address, event};

#[event(bytemuck)]
#[repr(C)]
pub struct InitializedVault {
    pub authority: Address,
    pub mint: Address,
}
