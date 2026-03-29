use clap::Subcommand;
use serde_json::{json, Value};

use crate::client::BybitClient;
use crate::commands::helpers::{build_params, confirm, optional_param};
use crate::errors::{BybitError, BybitResult};
use crate::output::{print_output, OutputFormat};

#[derive(Debug, clap::Args)]
pub struct SubaccountArgs {
    #[command(subcommand)]
    pub command: SubaccountCommand,
}

#[derive(Debug, Subcommand)]
pub enum SubaccountCommand {
    /// List up to 10k subaccounts for the master account
    List,
    /// Paginated list for masters with more than 10k subaccounts
    ListAll {
        #[arg(long)]
        page_size: Option<u32>,
        #[arg(long)]
        next_cursor: Option<String>,
    },
    /// Show wallet types for the master account or selected sub UIDs
    WalletTypes {
        /// Comma-separated sub UIDs
        #[arg(long)]
        member_ids: Option<String>,
    },
    /// List all API keys for a specific sub UID
    ApiKeys {
        #[arg(long)]
        sub_member_id: String,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
    },
    /// Create a new subaccount
    Create {
        #[arg(long)]
        username: String,
        #[arg(long)]
        password: Option<String>,
        /// 1 = normal subaccount, 6 = custodial subaccount
        #[arg(long, default_value = "1", value_parser = clap::value_parser!(u8).range(1..=6))]
        member_type: u8,
        /// Enable quick login for the new subaccount
        #[arg(long)]
        quick_login: bool,
    },
    /// Delete a subaccount
    Delete {
        #[arg(long)]
        sub_member_id: String,
    },
    /// Freeze a subaccount (Master account only)
    Freeze {
        #[arg(long)]
        sub_member_id: String,
    },
    /// Unfreeze a subaccount (Master account only)
    Unfreeze {
        #[arg(long)]
        sub_member_id: String,
    },
}

pub async fn run(
    args: SubaccountArgs,
    client: &BybitClient,
    format: OutputFormat,
    force: bool,
) -> BybitResult<()> {
    let value: Value = match args.command {
        SubaccountCommand::List => {
            client
                .private_get("/v5/user/query-sub-members", &[])
                .await?
        }

        SubaccountCommand::ListAll {
            page_size,
            next_cursor,
        } => {
            let page_size_str = page_size.map(|value| value.to_string());
            let params = build_params([
                page_size_str
                    .as_ref()
                    .map(|value| ("pageSize", value.as_str())),
                optional_param("nextCursor", &next_cursor),
            ]);
            client.private_get("/v5/user/submembers", &params).await?
        }

        SubaccountCommand::WalletTypes { member_ids } => {
            let params = build_params([optional_param("memberIds", &member_ids)]);
            client
                .private_get("/v5/user/get-member-type", &params)
                .await?
        }

        SubaccountCommand::ApiKeys {
            sub_member_id,
            limit,
            cursor,
        } => {
            let limit_str = limit.map(|value| value.to_string());
            let params = build_params([
                Some(("subMemberId", sub_member_id.as_str())),
                limit_str.as_ref().map(|value| ("limit", value.as_str())),
                optional_param("cursor", &cursor),
            ]);
            client.private_get("/v5/user/sub-apikeys", &params).await?
        }

        SubaccountCommand::Create {
            username,
            password,
            member_type,
            quick_login,
        } => {
            if member_type != 1 && member_type != 6 {
                return Err(BybitError::Validation(
                    "--member-type must be 1 (normal) or 6 (custodial).".to_string(),
                ));
            }

            confirm(
                &format!("Create subaccount `{username}` with member type {member_type}?"),
                force,
            )?;

            let mut body = json!({
                "username": username,
                "memberType": member_type,
                "switch": if quick_login { 1 } else { 0 },
            });
            if let Some(password) = password {
                body["password"] = json!(password);
            }
            client
                .private_post("/v5/user/create-sub-member", &body)
                .await?
        }

        SubaccountCommand::Delete { sub_member_id } => {
            confirm(
                &format!("Delete subaccount `{sub_member_id}`? This cannot be undone."),
                force,
            )?;
            let body = json!({ "subMemberId": sub_member_id });
            client.private_post("/v5/user/del-submember", &body).await?
        }

        SubaccountCommand::Freeze { sub_member_id } => {
            confirm(&format!("Freeze subaccount {sub_member_id}?"), force)?;
            let body = json!({ "subMemberId": sub_member_id, "frozen": 1 });
            client
                .private_post("/v5/user/frozen-sub-uid", &body)
                .await?
        }

        SubaccountCommand::Unfreeze { sub_member_id } => {
            confirm(&format!("Unfreeze subaccount {sub_member_id}?"), force)?;
            let body = json!({ "subMemberId": sub_member_id, "frozen": 0 });
            client
                .private_post("/v5/user/frozen-sub-uid", &body)
                .await?
        }
    };

    print_output(&value, format);
    Ok(())
}
