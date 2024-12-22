use diesel::pg::Pg;
use diesel::prelude::*;
use diesel_async::async_connection_wrapper::AsyncConnectionWrapper;
use diesel_async::pooled_connection::mobc::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncConnection;
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use std::env;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

use crate::model::{CreatePenguinEncounter, PenaltyEnum, PenguinEncounter};

pub type DatabasePool = Pool<AsyncPgConnection>;

async fn run_migrations<A>(async_connection: A) -> Result<(), Box<dyn std::error::Error>>
where
    A: AsyncConnection<Backend = Pg> + 'static,
{
    let mut async_wrapper: AsyncConnectionWrapper<A> =
        AsyncConnectionWrapper::from(async_connection);

    tokio::task::spawn_blocking(move || {
        async_wrapper.run_pending_migrations(MIGRATIONS).unwrap();
    })
    .await
    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

pub async fn init() -> DatabasePool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(database_url);

    let pool = Pool::new(config);

    let mut tries = 0;

    loop {
        let conn = pool.get().await;
        match conn {
            Ok(conn) => {
                run_migrations(conn).await.unwrap();
                break;
            }
            Err(e) => {
                eprintln!("Failed to connect to database: {}", e);
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        tries += 1;
        if tries > 10 {
            panic!("Failed to connect to database after 10 tries");
        }
    }

    pool
}

pub async fn list_penguin_encounters(
    conn: &mut AsyncPgConnection,
) -> Result<Vec<PenguinEncounter>, diesel::result::Error> {
    use crate::schema::penguin_encounter::dsl;

    dsl::penguin_encounter.load(conn).await
}

pub async fn create_penguin_encounter(
    conn: &mut AsyncPgConnection,
    name: &str,
    location: &str,
    penalty: PenaltyEnum,
    date_time: chrono::NaiveDateTime,
) -> Result<PenguinEncounter, diesel::result::Error> {
    use crate::schema::penguin_encounter::dsl;

    let penguin_encounter = CreatePenguinEncounter {
        name,
        location,
        penalty,
        date_time,
    };

    diesel::insert_into(dsl::penguin_encounter)
        .values(&penguin_encounter)
        .returning(PenguinEncounter::as_returning())
        .get_result(conn)
        .await
}
