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
    #[msg("Lottery time un-elapsed")]
    LotteryTimeUnElapsed,
    #[msg("Not enough ANTcoin")]
    InsufficientFunds,
    #[msg("Not owner.")]
    NotOwner,
    #[msg("Invalid Lottery Token Account for admin.")]
    InvalidLotteryTokenAccount,
    #[msg("Uninitialized Account")]
    UninitializedAccount,
    #[msg("Already Confirm")]
    AlreadyConfirm,
    #[msg("Not Confirmed")]
    NotConfirmed,
    #[msg("Didn't Deposit")]
    NotDeposit,
    #[msg("Lottery not claimable")]
    LotteryNotClaimable
}
