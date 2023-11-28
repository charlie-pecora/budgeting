use anyhow::Result;
use sqlx::sqlite::SqlitePool;
use uuid::Uuid;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct AccountType {
    id: String,
    name: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Account {
    id: String,
    name: String,
    bank_name: String,
    type_name: String,
}

pub async fn get_account(db: &SqlitePool, id: &str) -> Result<Account> {
    let account = sqlx::query_as!(
        Account, 
        "
select a.id,
       a.name,
       b.name as bank_name,
       at.name as type_name
from accounts a
join account_types at on a.type_id = at.id
join banks b on a.bank_id = b.id
where a.id = ?;
        ", 
        id
    )
        .fetch_one(db)
        .await?;
    Ok(account)
}

pub async fn list_accounts(db: &SqlitePool, bank_id: &Option<String>) -> Result<Vec<Account>> {
    let accounts = match bank_id {
        Some(v) => {
            sqlx::query_as!(
                Account, 
                "
select a.id,
       a.name,
       b.name as bank_name,
       at.name as type_name
from accounts a
join account_types at on a.type_id = at.id
join banks b on a.bank_id = b.id
where b.id = ?;
                ",
                v
            )
                .fetch_all(db)
                .await?
        }
        None => {
            sqlx::query_as!(
                Account, 
                "
select a.id,
       a.name,
       b.name as bank_name,
       at.name as type_name
from accounts a
join account_types at on a.type_id = at.id
join banks b on a.bank_id = b.id;
                "
            )
                .fetch_all(db)
                .await?
        }
    };
    Ok(accounts)
}

pub async fn create_account(
    db: &SqlitePool,
    name: &str,
    bank_id: &str,
    type_id: &str,
) -> Result<Account> {
    let id = Uuid::now_v7().hyphenated().to_string();
    sqlx::query_as!(
        Account,
        "
insert into accounts (
    id,
    name,
    bank_id,
    type_id
) values (
    ?,
    ?,
    ?,
    ?
);
        ",
        id,
        name,
        bank_id,
        type_id,
    )
    .execute(db)
    .await?;
    return get_account(db, &id).await;
}

pub async fn list_account_types(db: &SqlitePool) -> Result<Vec<AccountType>> {
    let account_types = sqlx::query_as!(AccountType, "select * from account_types;")
        .fetch_all(db)
        .await?;
    Ok(account_types)
}

pub async fn create_account_type(db: &SqlitePool, name: &str) -> Result<Vec<AccountType>> {
    let id = Uuid::now_v7().hyphenated().to_string();
    let created = sqlx::query_as!(
        AccountType,
        "
insert into account_types (
    id,
    name
) values (
    ?,
    ?
) returning *;
        ",
        id,
        name,
    )
    .fetch_all(db)
    .await?;
    Ok(created)
}
