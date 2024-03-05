use anchor_lang::error_code;

#[error_code]
pub enum LotteryError {
    #[msg("Lottery is already closed.")]
    LotteryClosed,
    #[msg("Lottery is still open.")]
    LotteryOpen,
    #[msg("Wrong lottery Id.")]
    InvalidLotteryId,
    #[msg("Unauthorized attempt to open/close the lottery.")]
    UnauthorizedOwner,
    #[msg("User is not whitelisted.")]
    NotWhitelisted,
    #[msg("Not enough tickets")]
    NotEnoughTickets,
    #[msg("Lottery time elapsed")]
    LotteryTimeElapsed,
    #[msg("Not enough ANTcoin")]
    InsufficientFunds,
    #[msg("Not owner.")]
    NotOwner,
}
