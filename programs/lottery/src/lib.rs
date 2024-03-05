mod errors;
mod events;
mod instructions;
mod state;
mod constants;

use anchor_lang::prelude::*;
pub use instructions::*;

declare_id!("66Tx1vJSxakHpk8xc7RRoFmh5U7qsQ8PvKMtQ8GSisL1");


#[program]
pub mod lottery {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        rewards_breakdown: Vec<u64>,
        bump: u8
    ) -> Result<()> {
        instructions::initialize(
            ctx,
            rewards_breakdown,
            bump
        )
    }

    pub fn start_lottery(
        ctx: Context<StartLottery>,
        end_time: u64,
        ant_coin_amount_per_ticket: u64,
    ) -> Result<()> {
        instructions::start_lottery_handler(
            ctx,
            end_time,
            ant_coin_amount_per_ticket,
        )
    }

    pub fn close_lottery(ctx: Context<CloseLottery>, lottery_id: u64) -> Result<()> {
        instructions::close_lottery_handler(ctx, lottery_id)
    }

    pub fn buy_tickets(ctx: Context<BuyTickets>, lottery_id: u64, quantity: u64) -> Result<()> {
        instructions::buy_tickets_handler(ctx, lottery_id, quantity)
    }

    pub fn process_draw_final_number_and_make_lottery_claimable(
        ctx: Context<ProcessDrawFinalNumberAndMakeLotteryClaimable>,
        lottery_id: u64,
    ) -> Result<()> {
        instructions::process_draw_final_number_and_make_lottery_claimable_handler(ctx, lottery_id)
    }

}
