mod errors;
mod events;
mod instructions;
mod state;
mod constants;

use anchor_lang::prelude::*;
use instructions::*;

declare_id!("AtGHjJbiGW3wWoTvYPhw14zENaUR779oCngPrTzcMHj6");


#[program]
pub mod lottery {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        rewards_breakdown: Vec<u64>,
        bump: u8,
    ) -> Result<()> {
        instructions::initialize(
            ctx,
            rewards_breakdown,
            bump
        )
    }

    pub fn start_lottery(
        ctx: Context<StartLottery>,
        force: [u8; 32],
        end_time: u64,
        lottery_coin_amount_per_ticket: u64,
    ) -> Result<()> {
        instructions::start_lottery_handler(
            ctx,
            force,
            end_time,
            lottery_coin_amount_per_ticket,
        )
    }

    pub fn close_lottery(ctx: Context<CloseLottery>, lottery_id: u64) -> Result<()> {
        instructions::close_lottery_handler(ctx,lottery_id)
    }

    pub fn buy_tickets(ctx: Context<BuyTickets>, force: [u8; 32], lottery_id: u64, ticket_id: u64)-> Result<()> {
        instructions::buy_tickets_handler(ctx, force, lottery_id, ticket_id)
    }

    pub fn confirm_tickets(ctx: Context<ConfirmTickets>) -> Result<()> {
        instructions::confirm_tickets_handler(ctx)
    }

    pub fn claim_tickets(ctx: Context<ClaimTicket>, ticket_order: u8) -> Result<()> {
        instructions::claim_tickets_handler(ctx, ticket_order)
    }

    pub fn calculate_antc_for_lottery(ctx: Context<CalculateAntcAmountForLottery>, lottery_price: u64, antc_price: u64) -> Result<()> {
        instructions::calculate_antc_for_lottery(ctx, lottery_price, antc_price)
    }

    pub fn deposit_antc_for_lottery(ctx: Context<DepositAntcForLottery>, amount: u64) -> Result<()> {
        instructions::deposit_antc_for_lottery(ctx, amount)
    }
    pub fn process_draw_final_number_and_make_lottery_claimable(
        ctx: Context<ProcessDrawFinalNumberAndMakeLotteryClaimable>,
        lottery_id: u64,
    ) -> Result<()> {
        instructions::process_draw_final_number_and_make_lottery_claimable_handler(ctx, lottery_id)
    }

}
