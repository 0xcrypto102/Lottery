use crate::errors::LotteryError;
use crate::events::TicketsPurchase;
use crate::{constants::*};
use crate::state::{Lottery, GlobalState, LotteryTicket, Ticket};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, Mint,  TokenAccount, Transfer, transfer};

use std::time::{SystemTime, UNIX_EPOCH};
use orao_solana_vrf::program::OraoVrf;
use orao_solana_vrf::state::NetworkState;
use orao_solana_vrf::CONFIG_ACCOUNT_SEED;
use orao_solana_vrf::RANDOMNESS_ACCOUNT_SEED;
use orao_solana_vrf::state::Randomness;

// use mpl_token_metadata::types::DataV2;
use std::mem::size_of;
use orao_solana_vrf::cpi::accounts::{ Request };


//  &force =  &global_state.current_lottery_id.to_le_bytes(),&(lottery_ticket.total_ticket + 1).to_le_bytes(), buyer.key().as_ref() 

#[derive(Accounts)]
#[instruction(force: [u8; 32])]
pub struct BuyTickets<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(
        mut,
        seeds = [LOTTERY_STATE_SEED],
        bump = global_state.bump,
    )]
    pub global_state: Box<Account<'info, GlobalState>>,

    #[account(
        mut,
        seeds = [LOTTERY_START_SEED, &global_state.current_lottery_id.to_le_bytes()],
        bump
    )]
    pub lottery: Box<Account<'info, Lottery>>,

    #[account(
        init_if_needed,
        payer = buyer,
        seeds = [LOTTERY_TICKET_SEED, &global_state.current_lottery_id.to_le_bytes(), buyer.key().as_ref()],
        bump,
        space = 8 + size_of::<LotteryTicket>(),
    )]
    pub lottery_ticket: Box<Account<'info, LotteryTicket>>,
    
    #[account(
        init,
        payer = buyer,
        seeds = [TICKET_SEED, &global_state.current_lottery_id.to_le_bytes(), &(lottery_ticket.total_ticket + 1).to_le_bytes(), buyer.key().as_ref()],
        bump,
        space = 8 + size_of::<Ticket>(),
    )]
    pub ticket: Box<Account<'info, Ticket>>,

    pub token_for_lottery: Account<'info, Mint>,
    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>, 
    #[account(mut)]
    pub admin_lottery_token_account: Account<'info, TokenAccount>, // Admin Account
    // Oracle for generating random number
    
    /// This account is the current VRF request account, it'll be the `request` account in the CPI call.
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

impl<'info> BuyTickets<'info> {
    fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.buyer_token_account.to_account_info(),
                to: self.admin_lottery_token_account.to_account_info(),
                authority: self.buyer.to_account_info(),
            },
        )
    }
}

#[derive(Accounts)]
pub struct ConfirmTickets<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [LOTTERY_STATE_SEED],
        bump = global_state.bump,
    )]
    pub global_state: Box<Account<'info, GlobalState>>,

    #[account(
        mut,
        seeds = [LOTTERY_START_SEED, &global_state.current_lottery_id.to_le_bytes()],
        bump
    )]
    pub lottery: Box<Account<'info, Lottery>>,

    #[account(
        mut,
        seeds = [LOTTERY_TICKET_SEED, &global_state.current_lottery_id.to_le_bytes(), user.key().as_ref()],
        bump,
    )]
    pub lottery_ticket: Box<Account<'info, LotteryTicket>>,
    
    #[account(
        mut,
        seeds = [TICKET_SEED, &global_state.current_lottery_id.to_le_bytes(), &(lottery_ticket.total_ticket).to_le_bytes(), user.key().as_ref()],
        bump,
    )]
    pub ticket: Box<Account<'info, Ticket>>,
    /// CHECK:` doc comment explaining why no checks through types are necessary.
    #[account(
        seeds = [RANDOMNESS_ACCOUNT_SEED.as_ref(), ticket.force.as_ref()],
        bump,
        seeds::program = orao_solana_vrf::ID
    )]
    pub random: AccountInfo<'info>,
    /// CHECK:` doc comment explaining why no checks through types are necessary.
    #[account(
        seeds = [RANDOMNESS_ACCOUNT_SEED.as_ref(), lottery.force.as_ref()],
        bump,
        seeds::program = orao_solana_vrf::ID
    )]
    pub lottery_random: AccountInfo<'info>,
    pub system_program: Program<'info, System>
}


#[derive(Accounts)]
#[instruction(ticket_order: u8)]
pub struct ClaimTicket<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [LOTTERY_STATE_SEED],
        bump = global_state.bump,
    )]
    pub global_state: Box<Account<'info, GlobalState>>,

    #[account(
        mut,
        seeds = [LOTTERY_START_SEED, &global_state.current_lottery_id.to_le_bytes()],
        bump
    )]
    pub lottery: Box<Account<'info, Lottery>>,

    #[account(
        seeds = [LOTTERY_START_SEED, &(global_state.current_lottery_id-1).to_le_bytes()],
        bump,
    )]
    pub prev_lottery: Box<Account<'info, Lottery>>,

    #[account(
        mut,
        seeds = [LOTTERY_TICKET_SEED, &global_state.current_lottery_id.to_le_bytes(), user.key().as_ref()],
        bump,
    )]
    pub lottery_ticket: Box<Account<'info, LotteryTicket>>,
    
    #[account(
        mut,
        seeds = [TICKET_SEED, &global_state.current_lottery_id.to_le_bytes(), &(ticket_order).to_le_bytes(), user.key().as_ref()],
        bump,
    )]
    pub ticket: Box<Account<'info, Ticket>>,

    pub token_for_antc: Box<Account<'info, Mint>>,

    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>, // buyer token account with checks
  
    #[account(
        mut,
        token::mint = token_for_antc,
        token::authority = global_state,
    )]
    pub antc_token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK:` doc comment explaining why no checks through types are necessary.
    #[account(
        seeds = [RANDOMNESS_ACCOUNT_SEED.as_ref(), ticket.force.as_ref()],
        bump,
        seeds::program = orao_solana_vrf::ID
    )]
    pub random: AccountInfo<'info>,
    /// CHECK:` doc comment explaining why no checks through types are necessary.
    #[account(
        seeds = [RANDOMNESS_ACCOUNT_SEED.as_ref(), lottery.force.as_ref()],
        bump,
        seeds::program = orao_solana_vrf::ID
    )]
    pub lottery_random: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>
}


pub fn buy_tickets_handler(ctx: Context<BuyTickets>,  force: [u8; 32], lottery_id: u64, ticket_id: u64) -> Result<()> {
    let lottery = &mut ctx.accounts.lottery;
    let lottery_ticket = &mut ctx.accounts.lottery_ticket;
    let ticket = &mut ctx.accounts.ticket;
    let buyer = ctx.accounts.buyer.key;

    require!(
        lottery.end_time
            > ctx.accounts.clock.unix_timestamp.try_into().unwrap(),
        LotteryError::LotteryTimeElapsed
    );
    require!(
        lottery.status == 0,
        LotteryError::LotteryClosed
    );
    require_eq!(&lottery.id, &lottery_id, LotteryError::InvalidLotteryId);

    require_eq!(ctx.accounts.global_state.lottery_token_account, ctx.accounts.admin_lottery_token_account.key(), LotteryError::InvalidLotteryTokenAccount);

    let amount_ant_for_transfer = lottery.lottery_coin_amount_per_ticket;
    lottery.amount_collected_in_lottery_coin += amount_ant_for_transfer;
    // lottery.amount_collected_in_lottery_coin += 1;

    lottery_ticket.lottery_id = lottery_id;
    lottery_ticket.total_ticket += 1;
    lottery_ticket.owner = ctx.accounts.buyer.key();

    ticket.lottery_id = lottery_id;
    ticket.ticket_order = ticket_id;
    ticket.randomness = ctx.accounts.random.key();
    ticket.force = force;
    ticket.owner = ctx.accounts.buyer.key();
    ticket.confirmed = false;

    token::transfer(ctx.accounts.transfer_context(), amount_ant_for_transfer * 10_u64.pow(u32::try_from(ctx.accounts.token_for_lottery.decimals).unwrap()))?;

    // Request randomness.
    let cpi_program = ctx.accounts.vrf.to_account_info();
    let cpi_accounts = Request {
        payer: ctx.accounts.buyer.to_account_info(),
        network_state: ctx.accounts.config.to_account_info(),
        treasury: ctx.accounts.treasury.to_account_info(),
        request: ctx.accounts.random.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    orao_solana_vrf::cpi::request(cpi_ctx, force)?;

    emit!(TicketsPurchase {
        buyer: buyer.key(),
        // NOTE: change this
        lottery_id: lottery_id,
        // NOTE: change this
        // number_tickets: quantity
    });

    Ok(())
}


pub fn confirm_tickets_handler(ctx: Context<ConfirmTickets>) -> Result<()> {
    let accts = ctx.accounts;

    if accts.random.data_is_empty() {
        return Err(LotteryError::UninitializedAccount.into());
    }

    if accts.lottery_random.data_is_empty() {
        return Err(LotteryError::UninitializedAccount.into());
    }

    if accts.ticket.confirmed {
        return Err(LotteryError::AlreadyConfirm.into());
    }

    require!(
        accts.lottery.status == 0,
        LotteryError::LotteryClosed
    );

    let ticket_account = Randomness::try_deserialize(&mut &accts.random.data.borrow()[..])?;
    let lottery_account = Randomness::try_deserialize(&mut &accts.lottery_random.data.borrow()[..])?;


    let Some(randomness) = ticket_account.fulfilled() else { todo!() };;
    let Some(lottery_randomness) = lottery_account.fulfilled() else { todo!() };;
    let mut match_number = 0_u8;

    for index in 0..6 {
        // Get the value for the current index from ticket_account's randomness
        let ticket_value = get_value(randomness, index);
    
        // Get the value for the current index from lottery_account's randomness
        let lottery_value = get_value(lottery_randomness, index);
    
        // Compare the values obtained from both accounts
        if ticket_value == lottery_value {
            match_number += 1;
        } else {
            break;
        }
    }
    match_number = 3;
    
    match match_number {
        3_u8 => {
            accts.lottery.winner_match3 += 1;
        },
        4_u8 => {
            accts.lottery.winner_match4 += 1;
        },
        5_u8 => {
            accts.lottery.winner_match5 += 1;
        },
        6_u8 => {
            accts.lottery.winner_match6 += 1;
        },
        _=> {

        }
    }
    accts.ticket.confirmed = true;

    Ok(())
} 

fn get_value(randomness: &[u8; 64], index: u8) -> u8 {
    // Calculate the starting position based on the index
    let start_position = index as usize * size_of::<u64>();

    // Extract the bytes at the calculated position
    let value = randomness[start_position..(start_position + size_of::<u64>())]
        .try_into()
        .unwrap();

    // Convert the bytes into a u64 and perform the modulo operation
    (u64::from_le_bytes(value) % 10) as u8
}

pub fn claim_tickets_handler(ctx: Context<ClaimTicket>, ticket_order: u8) -> Result<()> {
    let accts = ctx.accounts;

    if accts.random.data_is_empty() {
        return Err(LotteryError::UninitializedAccount.into());
    }

    if accts.lottery_random.data_is_empty() {
        return Err(LotteryError::UninitializedAccount.into());
    }

    if !accts.ticket.confirmed {
        return Err(LotteryError::NotConfirmed.into());
    }

    require!(
        accts.lottery.status == 2,
        LotteryError::LotteryNotClaimable
    );

    let ticket_account = Randomness::try_deserialize(&mut &accts.random.data.borrow()[..])?;
    let lottery_account = Randomness::try_deserialize(&mut &accts.lottery_random.data.borrow()[..])?;


    let Some(randomness) = ticket_account.fulfilled() else { todo!() };;
    let Some(lottery_randomness) = lottery_account.fulfilled() else { todo!() };;
    let mut match_number = 0_u8;

    for index in 0..6 {
        // Get the value for the current index from ticket_account's randomness
        let ticket_value = get_value(randomness, index);
    
        // Get the value for the current index from lottery_account's randomness
        let lottery_value = get_value(lottery_randomness, index);
    
        // Compare the values obtained from both accounts
        if ticket_value == lottery_value {
            match_number += 1;
        } else {
            break;
        }
    }
    
    
    let (_, bump) = Pubkey::find_program_address(&[LOTTERY_STATE_SEED], ctx.program_id);
    let vault_seeds = &[LOTTERY_STATE_SEED, &[bump]];
    let signer = &[&vault_seeds[..]];
    match_number = 3;
    match match_number {
        3_u8 => {
            let cpi_context = CpiContext::new(
                accts.token_program.to_account_info(),
                Transfer {
                    from: accts.antc_token_account.to_account_info(),
                    to: accts.buyer_token_account.to_account_info(),
                    authority: accts.global_state.to_account_info(),
                },
            );

            let mut amount = 0;

            if accts.lottery.id == 1 {
                amount = (accts.lottery.amount_antc_for_deposit) / accts.lottery.winner_match3 as u64 * accts.global_state.rewards_breakdown.match3 / 100;
            } else {
                amount = (accts.lottery.amount_antc_for_deposit + accts.prev_lottery.remain_match3) / accts.lottery.winner_match3 as u64 * accts.global_state.rewards_breakdown.match3 / 100;
            }
            transfer(cpi_context.with_signer(signer), amount as u64)?;

          
        },
        4_u8 => {
            let cpi_context = CpiContext::new(
                accts.token_program.to_account_info(),
                Transfer {
                    from: accts.antc_token_account.to_account_info(),
                    to: accts.buyer_token_account.to_account_info(),
                    authority: accts.global_state.to_account_info(),
                },
            );

            let mut amount = 0;

            if accts.lottery.id == 1 {
                amount = (accts.lottery.amount_antc_for_deposit) / accts.lottery.winner_match4 as u64 * accts.global_state.rewards_breakdown.match4 / 100;
            } else {
                amount = (accts.lottery.amount_antc_for_deposit + accts.prev_lottery.remain_match4) / accts.lottery.winner_match4 as u64 * accts.global_state.rewards_breakdown.match4 / 100;
            }
            transfer(cpi_context.with_signer(signer), amount as u64)?;
        },
        5_u8 => {
            let cpi_context = CpiContext::new(
                accts.token_program.to_account_info(),
                Transfer {
                    from: accts.antc_token_account.to_account_info(),
                    to: accts.buyer_token_account.to_account_info(),
                    authority: accts.global_state.to_account_info(),
                },
            );

            let mut amount = 0;

            if accts.lottery.id == 1 {
                amount = (accts.lottery.amount_antc_for_deposit) / accts.lottery.winner_match5 as u64 * accts.global_state.rewards_breakdown.match5 / 100;
            } else {
                amount = (accts.lottery.amount_antc_for_deposit + accts.prev_lottery.remain_match5) / accts.lottery.winner_match5 as u64 * accts.global_state.rewards_breakdown.match5 / 100;
            }
            transfer(cpi_context.with_signer(signer), amount as u64)?;
        },
        6_u8 => {
            let cpi_context = CpiContext::new(
                accts.token_program.to_account_info(),
                Transfer {
                    from: accts.antc_token_account.to_account_info(),
                    to: accts.buyer_token_account.to_account_info(),
                    authority: accts.global_state.to_account_info(),
                },
            );

            let mut amount = 0;

            if accts.lottery.id == 1 {
                amount = (accts.lottery.amount_antc_for_deposit) / accts.lottery.winner_match6 as u64 * accts.global_state.rewards_breakdown.match6 / 100;
            } else {
                amount = (accts.lottery.amount_antc_for_deposit + accts.prev_lottery.remain_match6) / accts.lottery.winner_match6 as u64 * accts.global_state.rewards_breakdown.match6 / 100;
            }
            transfer(cpi_context.with_signer(signer), amount as u64)?;
        },
        _=> {

        }
    }
    accts.ticket.confirmed = true;

    Ok(())
} 
