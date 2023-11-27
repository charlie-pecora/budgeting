use anyhow::Result;
use sqlx::sqlite::SqlitePool;
use uuid::Uuid;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct NewBank {
    name: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Bank {
    id: String,
    name: String,
}

pub async fn list_banks(db: &SqlitePool) -> Result<Vec<Bank>> {
    let banks = sqlx::query_as!(
        Bank,
        "
        select * from banks;
        "
    )
    .fetch_all(db)
    .await?;
    Ok(banks)
}

pub async fn create_bank(db: &SqlitePool, name: &str) -> Result<Vec<Bank>> {
    let bank = NewBank {
        name: name.to_string(),
    };
    let new_bank = insert_bank(db, &bank).await?;
    Ok(new_bank)
}

pub async fn insert_bank(db: &SqlitePool, bank: &NewBank) -> Result<Vec<Bank>> {
    let id = Uuid::now_v7().hyphenated().to_string();
    let created = sqlx::query_as!(
        Bank,
        "
insert into banks (
    id,
    name
) values (
    ?,
    ?
) returning *;
        ",
        id,
        bank.name,
    )
    .fetch_all(db)
    .await?;
    Ok(created)
}
