use crate::events::LotteryOpen;
use crate::state::{Lottery, LotteryStatus};
use anchor_lang::prelude::{Pubkey, *};
use std::time::{SystemTime, UNIX_EPOCH};

// TODO give role to the pubkey that starts the lottery

#[derive(Accounts)]
pub struct StartLottery<'info> {
    // #[account(init, payer = user, space = 8 + 8 + 8 * 6 + 4 + 32 + 16)] // Adjusted space
    #[account(
        init, 
        payer = owner, 
        space = 1020
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
    rewards_breakdown: Vec<u64>,
    lottery_id: u64,
    ant_coin_amount_per_ticket: u64,
) -> Result<()> {
    let lottery = &mut ctx.accounts.lottery;
    // lottery.add_to_whitelist(whitelist_addresses);

    // Set other lottery parameters
    lottery.id = lottery_id;
    lottery.end_time = end_time;
    // lottery.rewards_breakdown = rewards_breakdown;
    lottery.status = LotteryStatus::Open;
    lottery.owner = *ctx.accounts.owner.key;
    lottery.ant_coin_amount_per_ticket = ant_coin_amount_per_ticket;

    emit!(LotteryOpen {
        lottery_id: lottery_id,
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
