use crate::events::Initialized;
use crate::state::{ GlobalState };
use crate::{constants::*};

use anchor_lang::prelude::{Pubkey, *};
use std::time::{SystemTime, UNIX_EPOCH};
use std::mem::size_of;

// TODO give role to the pubkey that starts the lottery

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct Initialize<'info> {
    // #[account(init, payer = user, space = 8 + 8 + 8 * 6 + 4 + 32 + 16)] // Adjusted space
    #[account(
        init, 
        seeds = [LOTTERY_STATE_SEED],
        bump,
        space = 8 + size_of::<GlobalState>(),
        payer = owner, 
    )] // TODO Adjusted space
    pub global_state: Account<'info, GlobalState>,
    // consider renaming the signer from user to owner because they start the lottery
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
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
