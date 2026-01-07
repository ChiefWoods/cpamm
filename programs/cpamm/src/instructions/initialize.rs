use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{Config, CONFIG_SEED, LP_SEED};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeArgs {
    pub seed: u64,
    pub locked: bool,
    pub fee: u16,
}

#[derive(Accounts)]
#[instruction(args: InitializeArgs)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = Config::DISCRIMINATOR.len() + Config::INIT_SPACE,
        seeds = [CONFIG_SEED, args.seed.to_le_bytes().as_ref()],
        bump,
    )]
    pub config: Account<'info, Config>,
    #[account(
        init,
        payer = authority,
        seeds = [LP_SEED, config.key().as_ref()],
        bump,
        mint::decimals = 6,
        mint::authority = config,
        mint::token_program = token_program,
    )]
    pub mint_lp: InterfaceAccount<'info, Mint>,
    #[account(mint::token_program = token_program)]
    pub mint_x: InterfaceAccount<'info, Mint>,
    #[account(mint::token_program = token_program)]
    pub mint_y: InterfaceAccount<'info, Mint>,
    #[account(
        init,
        payer = authority,
        associated_token::mint = mint_x,
        associated_token::authority = config,
        associated_token::token_program = token_program,
    )]
    pub vault_x: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init,
        payer = authority,
        associated_token::mint = mint_y,
        associated_token::authority = config,
        associated_token::token_program = token_program,
    )]
    pub vault_y: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl Initialize<'_> {
    pub fn handler(ctx: Context<Initialize>, args: InitializeArgs) -> Result<()> {
        ctx.accounts.config.set_inner(Config {
            seed: args.seed,
            locked: args.locked,
            bump: ctx.bumps.config,
            lp_bump: ctx.bumps.mint_lp,
            fee: args.fee,
            mint_x: ctx.accounts.mint_x.key(),
            mint_y: ctx.accounts.mint_y.key(),
            authority: ctx.accounts.authority.key(),
        });

        Ok(())
    }
}
