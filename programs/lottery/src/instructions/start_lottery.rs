use crate::events::LotteryOpen;
use crate::state::{Lottery, GlobalState};
use crate::{constants::*, errors::*};
use anchor_spl::token::{self, Token, Mint,  TokenAccount, Transfer};

use anchor_lang::prelude::{Pubkey, *};
use std::time::{SystemTime, UNIX_EPOCH};

use orao_solana_vrf::program::OraoVrf;
use orao_solana_vrf::state::NetworkState;
use orao_solana_vrf::CONFIG_ACCOUNT_SEED;
use orao_solana_vrf::RANDOMNESS_ACCOUNT_SEED;
use orao_solana_vrf::state::Randomness;

use std::mem::size_of;
use orao_solana_vrf::cpi::accounts::{ Request };


// TODO give role to the pubkey that starts the lottery

#[derive(Accounts)]
#[instruction(force: [u8; 32])]
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
    )] 
    pub lottery: Account<'info, Lottery>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    // This account is the current VRF request account, it'll be the `request` account in the CPI call.
    /// CHECK:
    #[account(
        mut,
        seeds = [RANDOMNESS_ACCOUNT_SEED.as_ref(), &force],
        bump,
        seeds::program = orao_solana_vrf::ID
    )]
    pub random: AccountInfo<'info>,

    /// CHECK:` doc comment explaining why no checks through types are necessary.
    #[account(mut)]
    pub treasury: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [CONFIG_ACCOUNT_SEED.as_ref()],
        bump,
        seeds::program = orao_solana_vrf::ID
    )]
    pub config: Box<Account<'info, NetworkState>>,
    pub vrf: Program<'info, OraoVrf>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn start_lottery_handler(
    ctx: Context<StartLottery>,
    force: [u8; 32],
    end_time: u64,
    lottery_coin_amount_per_ticket: u64,
) -> Result<()> {
    let accts = ctx.accounts;

    if accts.owner.key() != accts.global_state.owner {
        return Err(LotteryError::NotOwner.into());
    }

    accts.lottery.id = accts.global_state.current_lottery_id + 1;
    accts.lottery.end_time = end_time + accts.clock.unix_timestamp as u64;
    accts.lottery.status = 0;
    accts.lottery.owner = accts.owner.key();
    accts.lottery.lottery_coin_amount_per_ticket = lottery_coin_amount_per_ticket;
    accts.lottery.guess = 0;
    accts.lottery.force = force;
    accts.lottery.winner_match3 = 0;
    accts.lottery.winner_match4 = 0;
    accts.lottery.winner_match5 = 0;
    accts.lottery.winner_match6 = 0;
    accts.lottery.remain_match3 = 0;
    accts.lottery.remain_match4 = 0;
    accts.lottery.remain_match5 = 0;
    accts.lottery.remain_match6 = 0;

    accts.global_state.current_lottery_id += 1;

     // Request randomness.
     let cpi_program = accts.vrf.to_account_info();
     let cpi_accounts = Request {
         payer: accts.owner.to_account_info(),
         network_state: accts.config.to_account_info(),
         treasury: accts.treasury.to_account_info(),
         request: accts.random.to_account_info(),
         system_program: accts.system_program.to_account_info(),
     };
     let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
     orao_solana_vrf::cpi::request(cpi_ctx, force)?;
    
    Ok(())
}
