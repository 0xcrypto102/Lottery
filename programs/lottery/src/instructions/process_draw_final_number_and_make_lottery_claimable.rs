use crate::errors::LotteryError;
use crate::errors::LotteryError::NotOwner;
use crate::events::LotteryNumberDrawn;
use crate::state::{Lottery, LotteryStatus};
use anchor_lang::prelude::{Pubkey, *};

#[derive(Accounts)]
pub struct ProcessDrawFinalNumberAndMakeLotteryClaimable<'info> {
    #[account(init, payer = owner, space = 1020)] 
    pub lottery: Account<'info, Lottery>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn process_draw_final_number_and_make_lottery_claimable_handler(
    ctx: Context<ProcessDrawFinalNumberAndMakeLotteryClaimable>,
    lottery_id: u64,
) -> Result<()> {
    let lottery = &mut ctx.accounts.lottery;
    require_eq!(&lottery.owner, &ctx.accounts.owner.key(), NotOwner);
    require!(
        &lottery.status.equal_to(LotteryStatus::Closed),
        LotteryError::LotteryOpen
    );

    // TODO get final number
    // TODO get count winning tickets

    // Change the status of the lottery to 'Claimable'
    lottery.status = LotteryStatus::Claimable;

    emit!(LotteryNumberDrawn {
        lottery_id: lottery_id,
        // TODO change this
        final_number: 0,
        // TODO change this
        count_winning_tickets: 0,
    });

    Ok(())
}
