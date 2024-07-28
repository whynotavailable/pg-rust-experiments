use sqlx::postgres::types::PgHstore;
use sqlx::postgres::PgPoolOptions;
use sqlx::types::Uuid;

#[derive(sqlx::FromRow)]
struct Agent {
    key: Uuid,
    name: String,
    config: PgHstore,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(50)
        .connect("postgres://postgres:example@localhost/test")
        .await?;

    upsert_one(&pool).await?;
    get_one(&pool).await?;

    Ok(())
}

async fn upsert_one(pool: &sqlx::Pool<sqlx::Postgres>) -> Result<(), sqlx::Error> {
    let sql = "INSERT INTO Agent
                    (key, name, config) VALUES
                    ($1, $2, $3)
                    ON CONFLICT (key) DO UPDATE SET name = excluded.name, config = excluded.config;";

    let mut store = PgHstore::default();
    store.insert("jo".to_string(), Some("sally".to_string()));

    let mut tx = pool.begin().await?; // Most things outside batches should need transactions.

    sqlx::query(sql)
        .bind(Uuid::parse_str("7ed88f93-3b34-4ae8-a869-54d0444af991").unwrap())
        .bind("Dave".to_string())
        .bind(store)
        .execute(&mut *tx)
        .await?;

    tx.commit().await
}

async fn get_one(pool: &sqlx::Pool<sqlx::Postgres>) -> Result<(), sqlx::Error> {
    let row: Option<Agent> = sqlx::query_as("SELECT * FROM public.agent")
        .fetch_optional(pool)
        .await?;

    if row.is_some() {
        let row = row.unwrap();
        println!("{} {}", row.key, row.name);

        for (key, val) in row.config.into_iter() {
            println!("{} => {}", key, val.unwrap_or("nil".to_string()))
        }
    } else {
        println!("No Row")
    }

    Ok(())
}
