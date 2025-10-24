use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, Debug, PartialEq)]
pub struct Contributor {
    pub amount: [u8; 8],
}

impl Contributor {
    pub const LEN: usize = core::mem::size_of::<Contributor>();

    pub fn to_bytes(&self) -> Vec<u8> {
        bytemuck::bytes_of(self).to_vec()
    }
}
