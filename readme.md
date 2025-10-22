# A no-std typical Escrow on Solana with Pinocchio, liteSVM, bytemuck

A production-ready Solana escrow program built with modern tooling for maximum performance and safety.

## Features

- **No-std Rust** - Minimal runtime overhead
- **Pinocchio** - Ultra-lightweight Solana framework
- **liteSVM** - Fast, in-process testing without validators
- **bytemuck** - Safe, zero-copy data serialization
- **Type-safe** - Shared data structures between client and program

## Project Structure

```bash
.
├── Cargo.lock
├── Cargo.toml
├── readme.md
└── src
    ├── instructions
    │   ├── make.rs      # Create escrow and deposit tokens
    │   ├── mod.rs       # Instruction routing
    │   └── take.rs      # Complete the swap
    ├── lib.rs           # Program entrypoint
    ├── state
    │   ├── escrow.rs    # Escrow account state
    │   └── mod.rs
    └── tests
        ├── mod.rs       # Integration tests with liteSVM
        └── smod.rs      # Test utilities and setup
```

## How It Works

This is a simple token swap escrow:

1. **Make**: Alice deposits Token A into an escrow, specifying how much Token B she wants
2. **Take**: Bob deposits Token B and receives Token A, completing the swap

### Make Instruction

```rust
// Alice creates escrow: "I'll give 400 Token A for 100 Token B"
MakeData {
    make_amount: 400_000_000,  // Amount to give (Token A)
    take_amount: 100_000_000,  // Amount to receive (Token B)
}
```

The program:

- Creates a PDA escrow account to store swap details
- Creates a vault (ATA) owned by the escrow PDA
- Transfers Token A from Alice to the vault

### Take Instruction

```rust
// Bob accepts the swap
```

The program:

- Validates Bob has enough Token B
- Transfers Token B from Bob to Alice
- Transfers Token A from vault to Bob
- Closes the vault and escrow accounts
- Returns rent to Alice

## Quick Start

### Prerequisites

```bash
# Install Solana CLI tools
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build

```bash
cargo build-sbf
```

### Test

```bash
cargo test -- --nocapture
```

### Build and Test

```bash
cargo build-sbf && cargo test -- --nocapture
```

## Key Implementation Details

### Bytemuck for Safe Serialization

Client-side serialization with type safety:

```rust
#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, Debug, PartialEq)]
pub struct MakeData {
    pub take_amount: u64,
    pub make_amount: u64,
}

impl MakeData {
    pub fn to_bytes(&self) -> Vec<u8> {
        bytemuck::bytes_of(self).to_vec()
    }
}

// Client
let make_data = MakeData {
    make_amount: 400_000_000,
    take_amount: 100_000_000,
};
let bytes = make_data.to_bytes();

// Program
let ix_data = bytemuck::pod_read_unaligned::<MakeData>(data);
```

### Escrow State Management

Zero-copy account data access:

```rust
#[repr(C)]
pub struct Escrow {
    maker: [u8; 32],
    mint_a: [u8; 32],
    mint_b: [u8; 32],
    amount_to_receive: [u8; 8],
    amount_to_give: [u8; 8],
    pub bump: u8,
}

impl Escrow {
    pub const LEN: usize = core::mem::size_of::<Self>();

    pub fn from_account_info(account_info: &AccountInfo) -> Result<&mut Self, ProgramError> {
        let mut data = account_info.try_borrow_mut_data()?;
        Ok(unsafe { &mut *(data.as_mut_ptr() as *mut Self) })
    }
}
```

### Modular Instruction Handlers

Separate concerns to attend to borrow checker issues:

```rust
EscrowInstrctions::Take => {
    instructions::process_take_instruction(accounts, data)?;
    instructions::transfer_to_maker(accounts)?;
    instructions::transfer_to_taker(accounts)?
}
```

### liteSVM Testing

Fast, in-process testing without running a validator:

```rust
#[test]
pub fn test_make_instruction() {
    let (mut svm, state) = setup();
    make(&mut svm, &state).unwrap();

    let maker_ata = svm.get_account(&state.maker_ata_a).unwrap();
    let account = spl_token::state::Account::unpack(&maker_ata.data).unwrap();

    assert_eq!(account.amount, expected_amount);
}
```

## Common Gotchas

### Account Borrow Errors

Always scope your borrows before CPIs:

```rust
// ✅ Good
let amount = {
    let escrow = Escrow::from_account_info(account)?;
    escrow.amount()
}; // Borrow dropped here
transfer_cpi(amount)?;

// ❌ Bad
let escrow = Escrow::from_account_info(account)?;
transfer_cpi(escrow.amount())?; // Error: account already borrowed
```

### Struct Size Calculation

Always use `size_of` to avoid missing fields:

```rust
// ❌ Faulty - easy to forget fields
pub const LEN: usize = 32 + 32 + 32 + 8 + 8; // Forgot bump!

// ✅ Good - compiler calculates
pub const LEN: usize = core::mem::size_of::<Self>();
```

### Bytemuck Alignment

Use `pod_read_unaligned` for instruction data:

```rust
// ❌ May fail on unaligned data
let data = bytemuck::from_bytes::<MakeData>(bytes)?;

// ✅ Works with any alignment
let data = bytemuck::pod_read_unaligned::<MakeData>(bytes);
```

## Resources

- [Pinocchio Documentation](https://github.com/febo/pinocchio)
- [liteSVM](https://github.com/LiteSVM/litesvm)
- [bytemuck](https://docs.rs/bytemuck)
- [Solana Cookbook](https://solanacookbook.com)

**Built with ❤️ using Pinocchio, liteSVM, and bytemuck**
