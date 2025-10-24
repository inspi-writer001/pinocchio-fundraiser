pub mod admin_claim;
pub mod contribute;
pub mod intialize;
pub mod refund;

pub use admin_claim::*;
pub use contribute::*;
pub use intialize::*;
pub use refund::*;

// #[repr(u8)]
pub enum FundraisingInstructions {
    Initialize = 0,
    Contribute = 1,
    CheckContributions = 2,
    Refund = 3,
}

// - intialize
// - contribute
// - check_contributions
// - refund
impl TryFrom<&u8> for FundraisingInstructions {
    type Error = pinocchio::program_error::ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(FundraisingInstructions::Initialize),
            1 => Ok(FundraisingInstructions::Contribute),
            2 => Ok(FundraisingInstructions::CheckContributions),
            3 => Ok(FundraisingInstructions::Refund),
            _ => Err(pinocchio::program_error::ProgramError::InvalidInstructionData),
        }
    }
}
