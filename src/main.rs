use std::path::PathBuf;
use std::str::FromStr;

use budgeting::accounts::{create_account, create_account_type, list_account_types, list_accounts};
use budgeting::app;
use budgeting::banks::{create_bank, list_banks};
use budgeting::transactions::{list_transactions, load_transactions_from_file};
use clap::{Parser, Subcommand};
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[clap(long, env, default_value = "sqlite://local.db")]
    pub database_url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run,
    GetTransactions,
    LoadTransactions {
        #[arg(value_name = "FILE")]
        source: PathBuf,

        #[clap(long)]
        account_id: String,
    },
    GetBanks,
    CreateBank {
        #[clap(long)]
        name: String,
    },
    GetAccounts {
        #[clap(long)]
        bank_id: Option<String>,
    },
    CreateAccount {
        #[clap(long)]
        name: String,
        #[clap(long)]
        bank_id: String,
        #[clap(long)]
        type_id: String,
    },
    GetAccountTypes,
    CreateAccountType {
        #[clap(long)]
        name: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let db_options = SqliteConnectOptions::from_str(&cli.database_url)?.create_if_missing(true);

    let db = SqlitePool::connect_with(db_options).await.unwrap();
    sqlx::migrate!().run(&db).await?;

    match &cli.command {
        Commands::Run => {
            let _ = app::run(db);
        }
        Commands::GetTransactions => {
            let transactions = list_transactions(&db).await?;
            for transaction in transactions {
                println!("{}", serde_json::to_string(&transaction)?);
            }
        }
        Commands::LoadTransactions { source, account_id } => {
            let loaded_count = load_transactions_from_file(&db, account_id, source).await?;
            println!(
                "Loaded {} transactions from {:?} to account {:?}",
                loaded_count, source, account_id
            );
        }
        Commands::GetBanks => {
            let banks = list_banks(&db).await?;
            for bank in banks {
                println!("{}", serde_json::to_string(&bank)?);
            }
        }
        Commands::CreateBank { name } => {
            let banks = create_bank(&db, name).await?;
            for bank in banks {
                println!("{}", serde_json::to_string(&bank)?);
            }
        }
        Commands::GetAccounts { bank_id } => {
            let accounts = list_accounts(&db, bank_id).await?;
            for account in accounts {
                println!("{}", serde_json::to_string(&account)?);
            }
        }
        Commands::CreateAccount {
            name,
            bank_id,
            type_id,
        } => {
            let account = create_account(&db, name, bank_id, type_id).await?;
            println!("{}", serde_json::to_string(&account)?);
        }
        Commands::GetAccountTypes => {
            let account_types = list_account_types(&db).await?;
            for account_type in account_types {
                println!("{}", serde_json::to_string(&account_type)?);
            }
        }
        Commands::CreateAccountType { name } => {
            let account_types = create_account_type(&db, name).await?;
            for account_type in account_types {
                println!("{}", serde_json::to_string(&account_type)?);
            }
        }
    }
    Ok(())
}
