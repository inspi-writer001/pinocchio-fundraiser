use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, Debug, PartialEq)]
pub struct Fundraiser {
    pub maker: [u8; 32],
    pub mint_to_raise: [u8; 32],
    pub amount_to_raise: [u8; 8],
    pub current_amount: [u8; 8],
    pub time_started: [u8; 8],
    pub duration: [u8; 8],
    pub bump: [u8; 1],
}

impl Fundraiser {
    pub const LEN: usize = core::mem::size_of::<Fundraiser>();

    pub fn max_sendable(&self) -> u64 {
        10_000_000_000
    } // 10K usd
    pub fn min_sendable(&self) -> u64 {
        10_000_000
    } // 10 usd

    pub fn to_bytes(&self) -> Vec<u8> {
        bytemuck::bytes_of(self).to_vec()
    }
}
