use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        burn, transfer_checked, Burn, Mint, TokenAccount, TokenInterface, TransferChecked,
    },
};
use constant_product_curve::{ConstantProduct, XYAmounts};

use crate::{error::AMMError, Config, CONFIG_SEED, LP_SEED};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct WithdrawArgs {
    amount: u64,
    min_x: u64,
    min_y: u64,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        has_one = mint_x,
        has_one = mint_y,
        seeds = [CONFIG_SEED, config.seed.to_le_bytes().as_ref()],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,
    #[account(
        mut,
        seeds = [LP_SEED, config.key().as_ref()],
        bump = config.lp_bump,
    )]
    pub mint_lp: Box<InterfaceAccount<'info, Mint>>,
    #[account(mint::token_program = token_program)]
    pub mint_x: Box<InterfaceAccount<'info, Mint>>,
    #[account(mint::token_program = token_program)]
    pub mint_y: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config,
        associated_token::token_program = token_program,
    )]
    pub vault_x: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config,
        associated_token::token_program = token_program,
    )]
    pub vault_y: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_lp,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_lp: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_x,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_x: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_y,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_y: Box<InterfaceAccount<'info, TokenAccount>>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl Withdraw<'_> {
    fn transfer_tokens(ctx: &Context<Withdraw>, is_x: bool, amount: u64) -> Result<()> {
        let (from, to, mint, decimals) = match is_x {
            true => (
                ctx.accounts.vault_x.to_account_info(),
                ctx.accounts.user_x.to_account_info(),
                ctx.accounts.mint_x.to_account_info(),
                ctx.accounts.mint_x.decimals,
            ),
            false => (
                ctx.accounts.vault_y.to_account_info(),
                ctx.accounts.user_y.to_account_info(),
                ctx.accounts.mint_y.to_account_info(),
                ctx.accounts.mint_y.decimals,
            ),
        };

        let signer_seeds: &[&[&[u8]]] = &[&[
            CONFIG_SEED,
            &ctx.accounts.config.seed.to_le_bytes(),
            &[ctx.accounts.config.bump],
        ]];

        transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    authority: ctx.accounts.config.to_account_info(),
                    from,
                    to,
                    mint,
                },
                signer_seeds,
            ),
            amount,
            decimals,
        )
    }

    pub fn handler(ctx: Context<Withdraw>, args: WithdrawArgs) -> Result<()> {
        Config::invariant(&ctx.accounts.config)?;
        require_gt!(args.amount, 0, AMMError::InvalidAmount);
        require!(
            args.min_x != 0 && args.min_y != 0,
            AMMError::InvalidMinAmount
        );

        let XYAmounts {
            x: amount_x,
            y: amount_y,
        } = ConstantProduct::xy_withdraw_amounts_from_l(
            ctx.accounts.vault_x.amount,
            ctx.accounts.vault_y.amount,
            ctx.accounts.mint_lp.supply,
            args.amount,
            6,
        )
        .unwrap();

        require!(
            args.min_x <= amount_x && args.min_y <= amount_y,
            AMMError::SlippageExceeded
        );

        Withdraw::transfer_tokens(&ctx, true, amount_x)?;
        Withdraw::transfer_tokens(&ctx, false, amount_y)?;

        burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    authority: ctx.accounts.user.to_account_info(),
                    from: ctx.accounts.user_lp.to_account_info(),
                    mint: ctx.accounts.mint_lp.to_account_info(),
                },
            ),
            args.amount,
        )
    }
}
