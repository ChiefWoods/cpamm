use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        mint_to, transfer_checked, Mint, MintTo, TokenAccount, TokenInterface, TransferChecked,
    },
};
use constant_product_curve::ConstantProduct;

use crate::{error::AMMError, Config, CONFIG_SEED, LP_SEED};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct DepositArgs {
    amount: u64,
    max_x: u64,
    max_y: u64,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
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
        init_if_needed,
        payer = user,
        associated_token::mint = mint_lp,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_lp: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_x: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_y: Box<InterfaceAccount<'info, TokenAccount>>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl Deposit<'_> {
    fn transfer_tokens(ctx: &Context<Deposit>, is_x: bool, amount: u64) -> Result<()> {
        let (from, to, mint, decimals) = match is_x {
            true => (
                ctx.accounts.user_x.to_account_info(),
                ctx.accounts.vault_x.to_account_info(),
                ctx.accounts.mint_x.to_account_info(),
                ctx.accounts.mint_x.decimals,
            ),
            false => (
                ctx.accounts.user_y.to_account_info(),
                ctx.accounts.vault_y.to_account_info(),
                ctx.accounts.mint_y.to_account_info(),
                ctx.accounts.mint_y.decimals,
            ),
        };

        transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    authority: ctx.accounts.user.to_account_info(),
                    from,
                    to,
                    mint,
                },
            ),
            amount,
            decimals,
        )
    }

    pub fn handler(ctx: Context<Deposit>, args: DepositArgs) -> Result<()> {
        Config::invariant(&ctx.accounts.config)?;
        require_gt!(args.amount, 0, AMMError::InvalidAmount);

        let (amount_x, amount_y) = match ctx.accounts.mint_lp.supply == 0
            && ctx.accounts.vault_x.amount == 0
            && ctx.accounts.vault_y.amount == 0
        {
            true => (args.max_x, args.max_y),
            false => {
                let amounts = ConstantProduct::xy_deposit_amounts_from_l(
                    ctx.accounts.vault_x.amount,
                    ctx.accounts.vault_y.amount,
                    ctx.accounts.mint_lp.supply,
                    args.amount,
                    6,
                )
                .unwrap();

                (amounts.x, amounts.y)
            }
        };

        require!(
            amount_x <= args.max_x && amount_y <= args.max_y,
            AMMError::SlippageExceeded
        );

        Deposit::transfer_tokens(&ctx, true, amount_x)?;
        Deposit::transfer_tokens(&ctx, false, amount_y)?;

        let signer_seeds: &[&[&[u8]]] = &[&[
            CONFIG_SEED,
            &ctx.accounts.config.seed.to_le_bytes(),
            &[ctx.accounts.config.bump],
        ]];

        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    authority: ctx.accounts.config.to_account_info(),
                    mint: ctx.accounts.mint_lp.to_account_info(),
                    to: ctx.accounts.user_lp.to_account_info(),
                },
                signer_seeds,
            ),
            args.amount,
        )
    }
}
