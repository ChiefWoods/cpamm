use anchor_lang::prelude::*;

use crate::{error::AMMError, Config};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UpdateConfigArgs {
    pub locked: Option<bool>,
    pub fee: Option<u16>,
    pub authority: Option<Pubkey>,
}

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    pub authority: Signer<'info>,
    #[account(
        mut,
        has_one = authority @ AMMError::InvalidConfigAuthority,
    )]
    pub config: Account<'info, Config>,
}

impl UpdateConfig<'_> {
    pub fn update_config(ctx: Context<UpdateConfig>, args: UpdateConfigArgs) -> Result<()> {
        if let Some(locked) = args.locked {
            ctx.accounts.config.locked = locked;
        }

        if let Some(fee) = args.fee {
            ctx.accounts.config.fee = fee;
        }

        if let Some(authority) = args.authority {
            ctx.accounts.config.authority = authority;
        }

        Ok(())
    }
}
