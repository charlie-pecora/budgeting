use std::{fs::File, path::PathBuf, str::FromStr};

use anyhow::Result;
use chrono::naive::NaiveDate;
use sqlx::sqlite::SqlitePool;
use uuid::Uuid;

#[derive(serde::Serialize, serde::Deserialize, Debug, sqlx::Type)]
pub struct NewTransaction {
    transaction_date: NaiveDate,
    description: String,
    amount_cents: i64,
    status: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, sqlx::Type, Clone)]
pub struct Transaction {
    id: String,
    account_name: String,
    transaction_date: NaiveDate,
    description: String,
    amount_cents: i64,
    status: String,
}

pub async fn get_transaction(db: &SqlitePool, id: &str) -> Result<Transaction> {
    let transaction = sqlx::query_as!(
        Transaction,
        "
select 
    t.id, 
    a.name as account_name, 
    t.transaction_date, 
    t.description,
    t.amount_cents,
    t.status 
from transactions t
join accounts a on t.account_id = a.id
where t.id = ?;
        ",
        id,
    )
    .fetch_one(db)
    .await?;
    Ok(transaction)
}

pub async fn list_transactions(db: &SqlitePool) -> Result<Vec<Transaction>> {
    let transactions = sqlx::query_as!(
        Transaction,
        "
select 
    t.id, 
    a.name as account_name, 
    t.transaction_date, 
    t.description,
    t.amount_cents,
    t.status 
from transactions t
join accounts a on t.account_id = a.id;
        "
    )
    .fetch_all(db)
    .await?;
    Ok(transactions)
}

pub async fn load_transactions_from_file(
    db: &SqlitePool,
    account_id: &str,
    source: &PathBuf,
) -> Result<u32> {
    let file = File::open(source)?;
    let mut reader = csv::Reader::from_reader(file);

    let mut count: u32 = 0;

    for row in reader.records() {
        let record = row?;
        let transaction = NewTransaction {
            transaction_date: NaiveDate::parse_from_str(&record[0], "%Y-%m-%d")?,
            description: record[2].to_string(),
            amount_cents: parse_cents(&record[4])?,
            status: record[5].to_string(),
        };
        insert_transaction(db, &transaction, account_id).await?;
        println!("{:?}", transaction);
        count += 1;
    }
    Ok(count)
}

fn parse_cents(cents_str: &str) -> Result<i64> {
    // break down dollars and cents
    let destructured: Vec<&str> = cents_str.split('.').collect();
    let mut dollars_str = "0".to_string();
    let mut cents_str = "0".to_string();
    if destructured.len() == 2 {
        dollars_str = destructured[0].to_string();
        cents_str = destructured[1].to_string();
        while cents_str.len() < 2 {
            cents_str = cents_str + "0";
        }
    } else if destructured.len() == 1 {
        dollars_str = destructured[0].to_string();
    }

    // compute cents
    let dollars = match i64::from_str(&dollars_str) {
        Ok(v) => v,
        Err(e) => {
            println!("{}, {}", &dollars_str, e);
            0
        }
    };
    let mut _cents = match i64::from_str(&cents_str) {
        Ok(v) => v,
        Err(e) => {
            println!("{}, {}", &cents_str, e);
            0
        }
    };
    if dollars_str.len() > 0 && &dollars_str[..1] == "-" {
        _cents = -1 * _cents;
    }
    let cents = dollars * 100 + _cents;
    Ok(cents)
}

pub async fn insert_transaction(
    db: &SqlitePool,
    transaction: &NewTransaction,
    account_id: &str,
) -> Result<Transaction> {
    let id = Uuid::now_v7().hyphenated().to_string();
    sqlx::query_as!(
        Transaction,
        "
insert into transactions (
    id,
    account_id,
    transaction_date,
    description,
    amount_cents,
    status
) values (
    ?,
    ?,
    ?,
    ?,
    ?,
    ?
)
        ",
        id,
        account_id,
        transaction.transaction_date,
        transaction.description,
        transaction.amount_cents,
        transaction.status,
    )
    .execute(db)
    .await?;
    return get_transaction(db, &id).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("0", 0)]
    #[case("1.0", 100)]
    #[case("0.23", 23)]
    #[case("12.01", 1201)]
    #[case("-12.01", -1201)]
    #[case("-0.01", -1)]
    #[case("12", 1200)]
    #[case("-12", -1200)]
    #[case(".2", 20)]
    #[case("-.02", -2)]
    #[case("2", 200)]
    #[case("-2", -200)]
    fn test_parse_cents(#[case] input: &str, #[case] expected: i64) -> Result<()> {
        let result_cents = parse_cents(input)?;
        assert_eq!(result_cents, expected);
        Ok(())
    }
}
