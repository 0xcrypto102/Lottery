use crate::errors::LotteryError;
use crate::events::TicketsPurchase;
use crate::state::{Lottery, LotteryStatus, Ticket};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Accounts)]
pub struct BuyTickets<'info> {
    #[account(mut)]
    pub lottery: Account<'info, Lottery>,
    // TODO change space
    #[account(init, payer = buyer, space = 8 + 42)] // Adjust space for your Ticket struct
    pub ticket: Account<'info, Ticket>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>, // buyer token account with checks
    #[account(mut)]
    pub admin_lottery_token_account: Account<'info, TokenAccount>, // Admin Account
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

pub fn buy_tickets_handler(ctx: Context<BuyTickets>, lottery_id: u64, quantity: u64) -> Result<()> {
    let lottery = &mut ctx.accounts.lottery;
    let buyer = ctx.accounts.buyer.key;

    require!(quantity > 0, LotteryError::NotEnoughTickets);
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
    // require!(
    //     &lottery.whitelist.contains(buyer),
    //     LotteryError::NotWhitelisted
    // );

    let amount_ant_for_transfer = lottery.ant_coin_amount_per_ticket * quantity;
    lottery.amount_collected_in_antcoin += amount_ant_for_transfer;

    let mut rng = rand::thread_rng();

    let random_number = rng.gen_range(1000000..1999999);

    for _ in 0..quantity {
        let ticket = &mut ctx.accounts.ticket;
        // ticket.number = lottery.current_ticket_id;
        ticket.number = random_number;
        ticket.owner = *ctx.accounts.buyer.key;

        lottery.current_ticket_id += 1;
    }

    token::transfer(ctx.accounts.transfer_context(), amount_ant_for_transfer)?;

    emit!(TicketsPurchase {
        buyer: buyer.key(),
        // NOTE: change this
        lottery_id: lottery_id,
        // NOTE: change this
        number_tickets: quantity
    });

    Ok(())
}
