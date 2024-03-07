use crate::errors::LotteryError;
use crate::errors::LotteryError::NotOwner;
use crate::events::LotteryNumberDrawn;
use crate::state::{Lottery, LotteryStatus, GlobalState};
use crate::{constants::*};
use anchor_lang::prelude::{Pubkey, *};
use anchor_spl::{
    token::{Mint, Token, TokenAccount,Transfer, transfer},
};

#[derive(Accounts)]
pub struct CalculateAntcAmountForLottery<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [LOTTERY_STATE_SEED],
        bump = global_state.bump,
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        mut,
        seeds = [LOTTERY_START_SEED, &global_state.current_lottery_id.to_le_bytes()],
        bump,
    )]
    pub lottery: Account<'info, Lottery>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositAntcForLottery<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [LOTTERY_STATE_SEED],
        bump = global_state.bump,
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        mut,
        seeds = [LOTTERY_START_SEED, &global_state.current_lottery_id.to_le_bytes()],
        bump,
    )]
    pub lottery: Account<'info, Lottery>,

    pub token_for_antc: Box<Account<'info, Mint>>,

    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>, // buyer token account with checks
  
    #[account(
        mut,
        token::mint = token_for_antc,
        token::authority = global_state,
    )]
    pub antc_token_account: Box<Account<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}
impl<'info> DepositAntcForLottery<'info> {
    fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.buyer_token_account.to_account_info(),
                to: self.antc_token_account.to_account_info(),
                authority: self.owner.to_account_info(),
            },
        )
    }
}


#[derive(Accounts)]
pub struct ProcessDrawFinalNumberAndMakeLotteryClaimable<'info> {
    #[account(init, payer = owner, space = 1020)] 
    pub lottery: Account<'info, Lottery>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn calculate_antc_for_lottery(
    ctx: Context<CalculateAntcAmountForLottery>,
    price: u64,
) -> Result<()> {
    
}

pub fn deposit_atc_for_lottery(
    ctx: Context<DepositAntcForLottery>,
    amount: u64
) -> Result<()> {
    let accts = ctx.accounts;
    require_eq!(amount, accts.lottery.amount)
    token::transfer(ctx.accounts.transfer_context(), amount)?;
    Ok(())
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
