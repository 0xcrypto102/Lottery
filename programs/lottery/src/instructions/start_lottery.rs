use crate::events::LotteryOpen;
use crate::state::{Lottery, GlobalState, LotteryStatus};
use crate::{constants::*, errors::*};

use anchor_lang::prelude::{Pubkey, *};
use std::time::{SystemTime, UNIX_EPOCH};

// use mpl_token_metadata::types::DataV2;
use std::mem::size_of;


// TODO give role to the pubkey that starts the lottery

#[derive(Accounts)]
pub struct StartLottery<'info> {
    #[account(
        mut,
        seeds = [LOTTERY_STATE_SEED],
        bump = global_state.bump,
    )]
    pub global_state: Account<'info, GlobalState>,

    // #[account(init, payer = user, space = 8 + 8 + 8 * 6 + 4 + 32 + 16)] // Adjusted space
    #[account(
        init, 
        seeds = [LOTTERY_START_SEED, &(global_state.current_lottery_id + 1).to_le_bytes()],
        bump,
        space = 8 + size_of::<Lottery>(),
        payer = owner, 
    )] // TODO Adjusted space
    pub lottery: Account<'info, Lottery>,
    // consider renaming the signer from user to owner because they start the lottery
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn start_lottery_handler(
    ctx: Context<StartLottery>,
    end_time: u64,
    lottery_coin_amount_per_ticket: u64,
) -> Result<()> {
    let accts = ctx.accounts;

    if accts.owner.key() != accts.global_state.owner {
        return Err(LotteryError::NotOwner.into());
    }

    accts.lottery.id = accts.global_state.current_lottery_id + 1;
    accts.lottery.end_time = end_time;
    accts.lottery.status = LotteryStatus::Open;
    accts.lottery.owner = accts.owner.key();
    accts.lottery.lottery_coin_amount_per_ticket = lottery_coin_amount_per_ticket;

    emit!(LotteryOpen {
        lottery_id: accts.lottery.id ,
        start_time: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        end_time: end_time,
        // TODO change
        first_ticket_id: 0,
        // TODO change
        injection_amount: 0
    });
    Ok(())
}
