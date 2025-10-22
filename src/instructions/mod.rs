pub mod make;
pub mod take;

pub use make::*;
pub use take::*;

pub mod intialize;
pub use intialize::*;

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
