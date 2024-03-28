use crate::errors::LotteryError;
use crate::events::LotteryClose;
use crate::state::{Lottery};
use anchor_lang::prelude::*;

use orao_solana_vrf::program::OraoVrf;
use orao_solana_vrf::state::NetworkState;
use orao_solana_vrf::CONFIG_ACCOUNT_SEED;
use orao_solana_vrf::RANDOMNESS_ACCOUNT_SEED;
use orao_solana_vrf::state::Randomness;

// use mpl_token_metadata::types::DataV2;
use std::mem::size_of;
use orao_solana_vrf::cpi::accounts::{ Request };


#[derive(Accounts)]
pub struct CloseLottery<'info> {
    #[account(mut, has_one = owner)]
    pub lottery: Account<'info, Lottery>,
    /// CHECK: This is only used for verification
    #[account(mut)]
    pub owner: Signer<'info>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn close_lottery_handler(ctx: Context<CloseLottery>, lottery_id: u64) -> Result<()> {
    let lottery = &mut ctx.accounts.lottery;

    require!(
        lottery.status == 0,
        LotteryError::LotteryClosed
    );
    require_eq!(&lottery.id, &lottery_id, LotteryError::InvalidLotteryId);
    require_eq!(
        &lottery.owner,
        ctx.accounts.owner.key,
        LotteryError::UnauthorizedOwner
    );

    require!(
        lottery.end_time
            < ctx.accounts.clock.unix_timestamp.try_into().unwrap(),
        LotteryError::LotteryTimeUnElapsed
    );

    // Additional logic for closing the lottery (e.g., finalizing winning number)

    // Change the status of the lottery to 'Closed'
    lottery.status = 1;
    
    emit!(LotteryClose {
        lottery_id: lottery_id,
        // TODO change this
        first_ticket_id_next_lottery: 0
    });

    Ok(())
}
