use crate::errors::LotteryError;
use crate::events::TicketsPurchase;
use crate::{constants::*};
use crate::state::{Lottery, LotteryStatus, GlobalState, LotteryTicket};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, Mint,  TokenAccount, Transfer};
use rand::Rng;
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
    pub global_state: Account<'info, GlobalState>,

    #[account(
        mut,
        seeds = [LOTTERY_START_SEED, &global_state.current_lottery_id.to_le_bytes()],
        bump
    )]
    pub lottery: Account<'info, Lottery>,

    #[account(
        init_if_needed,
        payer = buyer,
        seeds = [LOTTERY_TICKET_SEED, &global_state.current_lottery_id.to_le_bytes(), buyer.key().as_ref()],
        bump,
        space = 8 + size_of::<LotteryTicket>(),
    )]
    pub lottery_ticket: Account<'info, LotteryTicket>,

    pub token_for_lottery: Account<'info, Mint>,
    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>, 
    #[account(mut)]
    pub admin_lottery_token_account: Account<'info, TokenAccount>, // Admin Account
    // Oracle for generating random number
    
    /// CHECK:` doc comment explaining why no checks through types are necessary.
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

pub fn buy_tickets_handler(ctx: Context<BuyTickets>, lottery_id: u64, force: [u8; 32]) -> Result<()> {
    let lottery = &mut ctx.accounts.lottery;
    let buyer = ctx.accounts.buyer.key;

    require!(
        lottery.end_time
            < SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        LotteryError::LotteryTimeElapsed
    );
    require!(
        &lottery.status.equal_to(LotteryStatus::Open),
        LotteryError::LotteryClosed
    );
    require_eq!(&lottery.id, &lottery_id, LotteryError::InvalidLotteryId);

    require_eq!(ctx.accounts.global_state.lottery_token_account, ctx.accounts.admin_lottery_token_account.key(), LotteryError::InvalidLotteryTokenAccount);

    let amount_ant_for_transfer = lottery.lottery_coin_amount_per_ticket;
    // lottery.amount_collected_in_lottery_coin += amount_ant_for_transfer;
    lottery.amount_collected_in_lottery_coin += 1;

    token::transfer(ctx.accounts.transfer_context(), amount_ant_for_transfer)?;

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
