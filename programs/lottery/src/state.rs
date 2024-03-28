use anchor_lang::prelude::*;


#[account]
#[derive(Default)]
pub struct GlobalState {
    pub current_lottery_id: u64,
    pub rewards_breakdown: Reward, // Rewards distribution
    pub token_for_lottery: Pubkey, // the SPL token for lottery
    pub lottery_token_account: Pubkey, // Lottery token account for the reward pool
    pub token_for_antc: Pubkey, // ANTC
    pub antc_token_account: Pubkey, // ANTC PDA for the reward pool
    pub owner: Pubkey,               // The authority who can close the lottery
    pub bump: u8, // the bump for Lottery Account , I will use the PDA as Lottery
}

#[account]
#[derive(Default)]
pub struct Lottery {
    pub id: u64,                     // Unique ID for the lottery
    pub end_time: u64,               // Timestamp when the lottery ends
    pub status: u8,       // Status of the lottery
    pub owner: Pubkey,               // The authority who can close the lottery
    pub current_ticket_id: u64,      // Current ticket ID
    pub amount_collected_in_lottery_coin: u64, // amount collected
    pub lottery_coin_amount_per_ticket: u64,
    pub deposited: bool,
    pub amount_antc_for_deposit: u64,
    pub guess: u64,
    pub force: [u8; 32],
    pub winner_match3: u8,
    pub winner_match4: u8,
    pub winner_match5: u8,
    pub winner_match6: u8,
    pub remain_match3: u64,
    pub remain_match4: u64,
    pub remain_match5: u64,
    pub remain_match6: u64,
}

#[account] 
#[derive(Default)]
pub struct LotteryTicket {
    pub lottery_id: u64, // Uniquey lottery number
    pub total_ticket: u8, // total number of ticket that users buy in lottery
    pub owner: Pubkey, // buyer address
}

#[account]
#[derive(Default)]
pub struct Ticket {
    pub lottery_id: u64, // Unique lottery number
    pub ticket_order: u64,   // Unique ticket number
    pub randomness: Pubkey,  // random account
    pub force:  [u8; 32],
    pub owner: Pubkey, // The owner of the ticket
    pub confirmed: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq,Default, Debug)]
pub struct Reward {
    pub match3: u64,
    pub match4: u64,
    pub match5: u64,
    pub match6: u64,
}