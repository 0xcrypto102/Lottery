use crate::events::Initialized;
use crate::state::{ GlobalState };
use crate::{constants::*};

use anchor_spl::{
    token::{Mint, Token, TokenAccount},
};

use anchor_lang::prelude::{Pubkey, *};
use std::time::{SystemTime, UNIX_EPOCH};
use std::mem::size_of;

// TODO give role to the pubkey that starts the lottery

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct Initialize<'info> {
    #[account(
        init, 
        seeds = [LOTTERY_STATE_SEED],
        bump,
        space = 8 + size_of::<GlobalState>(),
        payer = owner, 
    )] // TODO Adjusted space
    pub global_state: Box<Account<'info, GlobalState>>,

    pub token_for_lottery: Box<Account<'info, Mint>>,

    #[account(mut)]
    pub lottery_token_account: Box<Account<'info, TokenAccount>>,// it should be owned by owner

    pub token_for_antc: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = owner,
        seeds = [TOKEN_VAULT_SEED, token_for_antc.key().as_ref()],
        bump,
        token::mint = token_for_antc,
        token::authority = global_state,
    )]
    pub antc_token_account: Box<Account<'info, TokenAccount>>,

    // consider renaming the signer from user to owner because they start the lottery
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn initialize(
    ctx: Context<Initialize>,
    rewards_breakdown: Vec<u64>,
    bump: u8
) -> Result<()> {
    let global_state = &mut ctx.accounts.global_state;
    // Set other lottery parameters
    global_state.current_lottery_id = 0;
    global_state.rewards_breakdown = rewards_breakdown.clone();
    global_state.token_for_lottery = ctx.accounts.token_for_lottery.key();
    global_state.lottery_token_account = ctx.accounts.lottery_token_account.key();
    global_state.token_for_antc = ctx.accounts.token_for_antc.key();
    global_state.antc_token_account = ctx.accounts.antc_token_account.key();
    global_state.owner = ctx.accounts.owner.key();
    global_state.bump = bump;

    emit!(Initialized {
        current_lottery_id: 0,
        rewards_breakdown:rewards_breakdown,
        // TODO change
        owner: ctx.accounts.owner.key()
    });
    Ok(())
}
