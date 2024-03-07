use anchor_lang::prelude::*;


#[account]
pub struct GlobalState {
    pub current_lottery_id: u64,
    pub rewards_breakdown: Vec<u64>, // Rewards distribution
    pub token_for_lottery: Pubkey, // the SPL token for lottery
    pub lottery_token_account: Pubkey, // Lottery token account for the reward pool
    pub owner: Pubkey,               // The authority who can close the lottery
    pub bump: u8, // the bump for Lottery Account , I will use the PDA as Lottery
}

#[account]
pub struct Lottery {
    pub id: u64,                     // Unique ID for the lottery
    pub end_time: u64,               // Timestamp when the lottery ends
    pub status: LotteryStatus,       // Status of the lottery
    pub owner: Pubkey,               // The authority who can close the lottery
    pub current_ticket_id: u64,      // Current ticket ID
    pub amount_collected_in_lottery_coin: u64, // amount collected
    pub lottery_coin_amount_per_ticket: u64,
}

#[account] 
pub struct LotteryTicket {
    pub lottery_id: u64, // Uniquey lottery number
    pub total_ticket: u8, // total number of ticket that users buy in lottery
    pub owner: Pubkey, // buyer address
}

// #[account]
// pub struct Ticket {
//     pub lottery_id: u64, // Unique lottery number
//     pub ticket_order: u64,   // Unique ticket number
//     pub owner: Pubkey, // The owner of the ticket
// }

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub enum LotteryStatus {
    Open,
    Closed,
    Claimable,
}

impl LotteryStatus {
    pub fn equal_to(&self, status: LotteryStatus) -> bool {
        *self == status
    }
}
