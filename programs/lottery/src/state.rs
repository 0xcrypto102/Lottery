use anchor_lang::prelude::*;

#[account]
pub struct Lottery {
    pub id: u64,                     // Unique ID for the lottery
    pub end_time: u64,               // Timestamp when the lottery ends
    pub rewards_breakdown: Vec<u64>, // Rewards distribution
    pub status: LotteryStatus,       // Status of the lottery
    pub owner: Pubkey,               // The authority who can close the lottery
    pub current_ticket_id: u64,      // Current ticket ID
    // I could optimize this later with a merkle tree or something
    pub whitelist: Vec<Pubkey>, // List of addresses allowed to buy tickets
    pub amount_collected_in_antcoin: u64, // amount collected
    pub ant_coin_amount_per_ticket: u64,
}

impl Lottery {
    pub fn contains(&self, address: &Pubkey) -> bool {
        self.whitelist.contains(address)
    }

    // TODO work on error handling
    pub fn add_to_whitelist(&mut self, addresses: Vec<Pubkey>) {
        let mut whitelist: Vec<Pubkey> = vec![];
        for address in addresses {
            whitelist.push(address);
        }
        self.whitelist = whitelist;
    }
}

#[account]
pub struct Ticket {
    pub number: u64,   // Unique ticket number
    pub owner: Pubkey, // The owner of the ticket
}

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
