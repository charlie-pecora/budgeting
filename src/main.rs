use std::path::PathBuf;
use std::str::FromStr;

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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let db_options = SqliteConnectOptions::from_str(&cli.database_url)?.create_if_missing(true);

    let db = SqlitePool::connect_with(db_options).await.unwrap();
    sqlx::migrate!().run(&db).await?;

    match &cli.command {
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
            let new_bank = create_bank(&db, name).await?;
            println!("{}", serde_json::to_string(&new_bank)?);
        }
    }
    Ok(())
}
