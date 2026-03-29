use clap::Subcommand;

use crate::client::BybitClient;
use crate::commands::asset::{run as run_asset, AssetArgs, AssetCommand};
use crate::errors::BybitResult;
use crate::output::OutputFormat;

#[derive(Debug, clap::Args)]
pub struct FundingArgs {
    #[command(subcommand)]
    pub command: FundingCommand,
}

#[derive(Debug, Subcommand)]
pub enum FundingCommand {
    /// Query coin info (networks, limits, chain support)
    CoinInfo {
        #[arg(long)]
        coin: Option<String>,
    },
    /// Query funding balances by account type
    Balance {
        #[arg(long, default_value = "FUND")]
        account_type: String,
        #[arg(long)]
        coin: Option<String>,
    },
    /// Query all balances across account types
    AllBalance {
        #[arg(long, default_value = "FUND")]
        account_type: String,
        #[arg(long)]
        coin: Option<String>,
        #[arg(long)]
        member_id: Option<String>,
    },
    /// Query a single funding coin balance
    AccountBalance {
        #[arg(long, default_value = "FUND")]
        account_type: String,
        #[arg(long)]
        member_id: Option<String>,
        #[arg(long)]
        coin: String,
        #[arg(long, default_value = "false")]
        with_bonus: bool,
    },
    /// List coins transferable between wallet types
    Transferable {
        #[arg(long)]
        from_account_type: String,
        #[arg(long)]
        to_account_type: String,
    },
    /// Transfer funds between wallets on the same UID
    Transfer {
        #[arg(long)]
        coin: String,
        #[arg(long)]
        amount: String,
        #[arg(long)]
        from_account_type: String,
        #[arg(long)]
        to_account_type: String,
    },
    /// Internal transfer history
    TransferHistory {
        #[arg(long)]
        coin: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        start: Option<u64>,
        #[arg(long)]
        end: Option<u64>,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
    },
    /// Universal transfer across UIDs
    SubTransfer {
        #[arg(long)]
        coin: String,
        #[arg(long)]
        amount: String,
        #[arg(long)]
        from_member_id: String,
        #[arg(long)]
        to_member_id: String,
        #[arg(long)]
        from_account_type: String,
        #[arg(long)]
        to_account_type: String,
    },
    /// Universal transfer history
    SubTransferHistory {
        #[arg(long)]
        coin: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        start: Option<u64>,
        #[arg(long)]
        end: Option<u64>,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
    },
    /// Get a deposit address
    DepositAddress {
        #[arg(long)]
        coin: String,
        #[arg(long)]
        chain_type: Option<String>,
    },
    /// Deposit history
    DepositHistory {
        #[arg(long)]
        coin: Option<String>,
        #[arg(long)]
        start: Option<u64>,
        #[arg(long)]
        end: Option<u64>,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
    },
    /// Withdraw funds to an external address
    Withdraw {
        #[arg(long)]
        coin: String,
        #[arg(long)]
        chain: String,
        #[arg(long)]
        address: String,
        #[arg(long)]
        amount: String,
        #[arg(long)]
        tag: Option<String>,
        #[arg(long, default_value = "UNIFIED")]
        account_type: String,
    },
    /// Withdrawal history
    WithdrawHistory {
        #[arg(long)]
        coin: Option<String>,
        #[arg(long)]
        withdraw_id: Option<String>,
        #[arg(long)]
        withdraw_type: Option<u8>,
        #[arg(long)]
        start: Option<u64>,
        #[arg(long)]
        end: Option<u64>,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
    },
    /// Cancel a pending withdrawal
    CancelWithdraw {
        #[arg(long)]
        id: String,
    },
}

pub async fn run(
    args: FundingArgs,
    client: &BybitClient,
    format: OutputFormat,
    force: bool,
) -> BybitResult<()> {
    match args.command {
        FundingCommand::CoinInfo { coin } => {
            run_asset(
                AssetArgs {
                    command: AssetCommand::CoinInfo { coin },
                },
                client,
                format,
                force,
            )
            .await
        }
        FundingCommand::Balance { account_type, coin } => {
            run_asset(
                AssetArgs {
                    command: AssetCommand::AllBalance {
                        account_type,
                        coin,
                        member_id: None,
                    },
                },
                client,
                format,
                force,
            )
            .await
        }
        FundingCommand::AllBalance {
            account_type,
            coin,
            member_id,
        } => {
            run_asset(
                AssetArgs {
                    command: AssetCommand::AllBalance {
                        account_type,
                        coin,
                        member_id,
                    },
                },
                client,
                format,
                force,
            )
            .await
        }
        FundingCommand::AccountBalance {
            account_type,
            member_id,
            coin,
            with_bonus,
        } => {
            run_asset(
                AssetArgs {
                    command: AssetCommand::AccountBalance {
                        account_type,
                        member_id,
                        coin,
                        with_bonus,
                    },
                },
                client,
                format,
                force,
            )
            .await
        }
        FundingCommand::Transferable {
            from_account_type,
            to_account_type,
        } => {
            run_asset(
                AssetArgs {
                    command: AssetCommand::Transferable {
                        from_account_type,
                        to_account_type,
                    },
                },
                client,
                format,
                force,
            )
            .await
        }
        FundingCommand::Transfer {
            coin,
            amount,
            from_account_type,
            to_account_type,
        } => {
            run_asset(
                AssetArgs {
                    command: AssetCommand::Transfer {
                        coin,
                        amount,
                        from_account_type,
                        to_account_type,
                    },
                },
                client,
                format,
                force,
            )
            .await
        }
        FundingCommand::TransferHistory {
            coin,
            status,
            start,
            end,
            limit,
            cursor,
        } => {
            run_asset(
                AssetArgs {
                    command: AssetCommand::TransferHistory {
                        coin,
                        status,
                        start,
                        end,
                        limit,
                        cursor,
                    },
                },
                client,
                format,
                force,
            )
            .await
        }
        FundingCommand::SubTransfer {
            coin,
            amount,
            from_member_id,
            to_member_id,
            from_account_type,
            to_account_type,
        } => {
            run_asset(
                AssetArgs {
                    command: AssetCommand::SubTransfer {
                        coin,
                        amount,
                        from_member_id,
                        to_member_id,
                        from_account_type,
                        to_account_type,
                    },
                },
                client,
                format,
                force,
            )
            .await
        }
        FundingCommand::SubTransferHistory {
            coin,
            status,
            start,
            end,
            limit,
            cursor,
        } => {
            run_asset(
                AssetArgs {
                    command: AssetCommand::SubTransferHistory {
                        coin,
                        status,
                        start,
                        end,
                        limit,
                        cursor,
                    },
                },
                client,
                format,
                force,
            )
            .await
        }
        FundingCommand::DepositAddress { coin, chain_type } => {
            run_asset(
                AssetArgs {
                    command: AssetCommand::DepositAddress { coin, chain_type },
                },
                client,
                format,
                force,
            )
            .await
        }
        FundingCommand::DepositHistory {
            coin,
            start,
            end,
            limit,
            cursor,
        } => {
            run_asset(
                AssetArgs {
                    command: AssetCommand::DepositHistory {
                        coin,
                        start,
                        end,
                        limit,
                        cursor,
                    },
                },
                client,
                format,
                force,
            )
            .await
        }
        FundingCommand::Withdraw {
            coin,
            chain,
            address,
            amount,
            tag,
            account_type,
        } => {
            run_asset(
                AssetArgs {
                    command: AssetCommand::Withdraw {
                        coin,
                        chain,
                        address,
                        amount,
                        tag,
                        account_type,
                    },
                },
                client,
                format,
                force,
            )
            .await
        }
        FundingCommand::WithdrawHistory {
            coin,
            withdraw_id,
            withdraw_type,
            start,
            end,
            limit,
            cursor,
        } => {
            run_asset(
                AssetArgs {
                    command: AssetCommand::WithdrawHistory {
                        coin,
                        withdraw_id,
                        withdraw_type,
                        start,
                        end,
                        limit,
                        cursor,
                    },
                },
                client,
                format,
                force,
            )
            .await
        }
        FundingCommand::CancelWithdraw { id } => {
            run_asset(
                AssetArgs {
                    command: AssetCommand::CancelWithdraw { id },
                },
                client,
                format,
                force,
            )
            .await
        }
    }
}
