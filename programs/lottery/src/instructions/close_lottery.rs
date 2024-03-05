use crate::errors::LotteryError;
use crate::events::LotteryClose;
use crate::state::{Lottery, LotteryStatus};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CloseLottery<'info> {
    #[account(mut, has_one = owner)]
    pub lottery: Account<'info, Lottery>,
    /// CHECK: This is only used for verification
    pub owner: AccountInfo<'info>,
}

pub fn close_lottery_handler(ctx: Context<CloseLottery>, lottery_id: u64) -> Result<()> {
    let lottery = &mut ctx.accounts.lottery;

    require!(
        &lottery.status.equal_to(LotteryStatus::Open),
        LotteryError::LotteryClosed
    );
    require_eq!(&lottery.id, &lottery_id, LotteryError::InvalidLotteryId);
    require_eq!(
        &lottery.owner,
        ctx.accounts.owner.key,
        LotteryError::UnauthorizedOwner
    );

    // Additional logic for closing the lottery (e.g., finalizing winning number)

    // Change the status of the lottery to 'Closed'
    lottery.status = LotteryStatus::Closed;

    emit!(LotteryClose {
        lottery_id: lottery_id,
        // TODO change this
        first_ticket_id_next_lottery: 0
    });

    Ok(())
}
