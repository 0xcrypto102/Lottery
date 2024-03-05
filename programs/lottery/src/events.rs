use anchor_lang::prelude::{Pubkey, *};
#[event]
pub struct AdminTokenRecovery {
    pub token: Pubkey,
    pub amount: u64,
}
#[event]
pub struct LotteryOpen {
    pub lottery_id: u64,
    pub start_time: u64,
    pub end_time: u64,
    pub first_ticket_id: u64,
    pub injection_amount: u64,
}
#[event]
pub struct LotteryClose {
    pub lottery_id: u64,
    pub first_ticket_id_next_lottery: u64,
}
#[event]
pub struct LotteryInjection {
    pub lottery_id: u64,
    pub injected_amount: u64,
}
#[event]
pub struct LotteryNumberDrawn {
    pub lottery_id: u64,
    pub final_number: u64,
    pub count_winning_tickets: u64,
}
#[event]
pub struct NewOperatorAndTreasuryAndInjectorAddresses {
    pub operator: Pubkey,
    pub injector: Pubkey,
}
#[event]
pub struct NewRandomGenerator {
    pub random_generator: Pubkey,
}
#[event]
pub struct TicketsPurchase {
    pub buyer: Pubkey,
    pub lottery_id: u64,
    pub number_tickets: u64,
}
#[event]
pub struct TicketsClaim {
    pub claimer: Pubkey,
    pub amount: u64,
    pub lottery_id: u64,
    pub number_tickets: u64,
}
