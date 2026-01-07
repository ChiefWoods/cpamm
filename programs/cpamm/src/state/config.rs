use crate::error::AMMError;
use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub seed: u64,
    pub locked: bool,
    pub bump: u8,
    pub lp_bump: u8,
    pub fee: u16,
    pub mint_x: Pubkey,
    pub mint_y: Pubkey,
    pub authority: Pubkey,
}

impl Config {
    pub fn invariant(&self) -> Result<()> {
        require!(self.locked == false, AMMError::PoolLocked);

        Ok(())
    }
}
