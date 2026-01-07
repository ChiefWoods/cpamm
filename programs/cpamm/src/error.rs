use anchor_lang::prelude::*;

#[error_code]
pub enum AMMError {
    #[msg("Invalid config authority")]
    InvalidConfigAuthority,
    #[msg("Pool is locked")]
    PoolLocked,
    #[msg("Amount must be greater than 0")]
    InvalidAmount,
    #[msg("Minimum amount of both tokens cannot be 0")]
    InvalidMinAmount,
    #[msg("Slippage exceeded")]
    SlippageExceeded,
}
