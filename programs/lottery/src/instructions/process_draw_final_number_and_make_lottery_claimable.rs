use crate::errors::LotteryError;
use crate::errors::LotteryError::NotOwner;
use crate::events::LotteryNumberDrawn;
use crate::state::{Lottery, GlobalState};
use crate::{constants::*};
use anchor_lang::prelude::{Pubkey, *};
// use anchor_spl::{
//     token::{Mint, Token, TokenAccount,Transfer, transfer},
// };
use orao_solana_vrf::program::OraoVrf;
use orao_solana_vrf::state::NetworkState;
use orao_solana_vrf::CONFIG_ACCOUNT_SEED;
use orao_solana_vrf::RANDOMNESS_ACCOUNT_SEED;
use orao_solana_vrf::state::Randomness;

// use mpl_token_metadata::types::DataV2;
use std::mem::size_of;
use orao_solana_vrf::cpi::accounts::{ Request };

use anchor_spl::{
    token::{mint_to, Mint, MintTo, Token, SetAuthority,TokenAccount, Burn, burn, Transfer, transfer},
};
use anchor_spl::token;

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

#[derive(Accounts)]
pub struct ProcessDrawFinalNumberAndMakeLotteryClaimable<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [LOTTERY_START_SEED, &global_state.current_lottery_id.to_le_bytes()],
        bump,
    )]
    pub lottery: Account<'info, Lottery>,

    #[account(
        init_if_needed,
        payer = owner,
        seeds = [LOTTERY_START_SEED, &(global_state.current_lottery_id-1).to_le_bytes()],
        bump,
        space = 8 + size_of::<Lottery>()
    )]
    pub prev_lottery: Account<'info, Lottery>,

    #[account(
        mut,
        seeds = [LOTTERY_STATE_SEED],
        bump = global_state.bump,
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(mut)]
    pub token_for_antc: Box<Account<'info, Mint>>,

    #[account(
        mut,
        token::mint = token_for_antc,
        token::authority = global_state,
    )]
    pub antc_token_account: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn calculate_antc_for_lottery(
    ctx: Context<CalculateAntcAmountForLottery>,
    lottery_price: u64,
    antc_price: u64,
) -> Result<()> {
    let accts = ctx.accounts;
    
    require_eq!(accts.lottery.owner, accts.owner.key(), NotOwner);

    let amount = 10_u64.pow(6) * accts.lottery.amount_collected_in_lottery_coin * lottery_price / antc_price;
    accts.lottery.amount_antc_for_deposit = amount;

    Ok(())
}

pub fn deposit_antc_for_lottery(
    ctx: Context<DepositAntcForLottery>,
    amount: u64
) -> Result<()> {
    let accts = ctx.accounts;
    require_eq!(amount, accts.lottery.amount_antc_for_deposit);
    require_eq!(false, accts.lottery.deposited);

    let cpi_ctx = CpiContext::new(
        accts.token_program.to_account_info(),
        Transfer {
            from: accts.buyer_token_account.to_account_info(),
            to: accts.antc_token_account.to_account_info(),
            authority: accts.owner.to_account_info(),
        },
    );
    transfer(cpi_ctx, amount)?;

    accts.lottery.deposited = true;
    Ok(())
}

pub fn process_draw_final_number_and_make_lottery_claimable_handler(
    ctx: Context<ProcessDrawFinalNumberAndMakeLotteryClaimable>,
    lottery_id: u64,
) -> Result<()> {
    let accts = ctx.accounts;
    require_eq!(accts.lottery.owner, accts.owner.key(), NotOwner);
    require!(
        accts.lottery.status == 1,
        LotteryError::LotteryOpen
    );
   
    accts.lottery.status = 2;

    require!(
        accts.lottery.deposited == true,
        LotteryError::NotDeposit
    );

    let (_, bump) = Pubkey::find_program_address(&[LOTTERY_STATE_SEED], ctx.program_id);
    let vault_seeds = &[LOTTERY_STATE_SEED, &[bump]];
    let signer = &[&vault_seeds[..]];


    if accts.lottery.winner_match3 == 0 {
        let cpi_context = CpiContext::new(
            accts.token_program.to_account_info(),
            Burn {
                mint: accts.token_for_antc.to_account_info(),
                from: accts.antc_token_account.to_account_info(),
                authority: accts.global_state.to_account_info(),
            },
        );
        let mut total_amount = 0;
        if accts.lottery.id == 1 {
            total_amount = accts.lottery.amount_antc_for_deposit  * accts.global_state.rewards_breakdown.match3 / 100 ;
        } else {
            total_amount = accts.lottery.amount_antc_for_deposit  * accts.global_state.rewards_breakdown.match3 / 100+ accts.prev_lottery.remain_match3;
        }

        let amount = total_amount * 4 / 10; 
        burn(cpi_context.with_signer(signer), amount as u64)?;
        accts.lottery.remain_match3 = total_amount - amount;
    } else {
        accts.lottery.remain_match3 = 0;
    }

    if accts.lottery.winner_match4 == 0 {
        let cpi_context = CpiContext::new(
            accts.token_program.to_account_info(),
            Burn {
                mint: accts.token_for_antc.to_account_info(),
                from: accts.antc_token_account.to_account_info(),
                authority: accts.global_state.to_account_info(),
            },
        );
        
        let mut total_amount = 0;
        if accts.lottery.id == 1 {
            total_amount = accts.lottery.amount_antc_for_deposit * accts.global_state.rewards_breakdown.match4 / 100;
        } else {
            total_amount = accts.lottery.amount_antc_for_deposit * accts.global_state.rewards_breakdown.match4 / 100 + accts.prev_lottery.remain_match4;
        }
        let amount = total_amount  * 4 / 10; 
        burn(cpi_context.with_signer(signer), amount as u64)?;
        accts.lottery.remain_match4 = total_amount - amount;
    } else {
        accts.lottery.remain_match4 = 0;
    }

    if accts.lottery.winner_match5 == 0 {
        let cpi_context = CpiContext::new(
            accts.token_program.to_account_info(),
            Burn {
                mint: accts.token_for_antc.to_account_info(),
                from: accts.antc_token_account.to_account_info(),
                authority: accts.global_state.to_account_info(),
            },
        );

        let mut total_amount = 0;
        if accts.lottery.id == 1 {
            total_amount = accts.lottery.amount_antc_for_deposit * accts.global_state.rewards_breakdown.match5 / 100;
        } else {
            total_amount = accts.lottery.amount_antc_for_deposit * accts.global_state.rewards_breakdown.match5 / 100 + accts.prev_lottery.remain_match5;
        }
        let amount = total_amount * 4 / 10; 
        burn(cpi_context.with_signer(signer), amount as u64)?;
        accts.lottery.remain_match5 = total_amount - amount;
    } else {
        accts.lottery.remain_match5 = 0;
    }

    if accts.lottery.winner_match6 == 0 {
        let cpi_context = CpiContext::new(
            accts.token_program.to_account_info(),
            Burn {
                mint: accts.token_for_antc.to_account_info(),
                from: accts.antc_token_account.to_account_info(),
                authority: accts.global_state.to_account_info(),
            },
        );

        let mut total_amount = 0;
        if accts.lottery.id == 1 {
            total_amount = accts.lottery.amount_antc_for_deposit * accts.global_state.rewards_breakdown.match6 / 100 ;
        } else {
            total_amount = accts.lottery.amount_antc_for_deposit * accts.global_state.rewards_breakdown.match6 / 100  + accts.prev_lottery.remain_match6;
        }
        let amount = total_amount * 4 / 10; 
        burn(cpi_context.with_signer(signer), amount as u64)?;
        accts.lottery.remain_match6 = total_amount - amount;
    } else {
        accts.lottery.remain_match6 = 0;
    }

    emit!(LotteryNumberDrawn {
        lottery_id: lottery_id,
        // TODO change this
        final_number: 0,
        // TODO change this
        count_winning_tickets: (accts.lottery.winner_match3 + accts.lottery.winner_match4 +accts.lottery.winner_match5 + accts.lottery.winner_match6) as u64
    });

    Ok(())
}
