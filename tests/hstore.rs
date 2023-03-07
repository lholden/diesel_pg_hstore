#![feature(test)]

extern crate diesel;
extern crate diesel_pg_hstore;
extern crate dotenv;
#[macro_use]
extern crate rstest;

use std::env;
use std::str;

use diesel::connection::SimpleConnection;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::Connection;

use diesel_pg_hstore::Hstore;

table! {
    use diesel::sql_types::*;
    use diesel_pg_hstore::Hstore;

    hstore_table {
        id -> Integer,
        store -> Hstore,
    }
}

#[derive(Insertable, Queryable, Identifiable, Debug, PartialEq)]
#[diesel(table_name = hstore_table)]
struct HasHstore {
    id: i32,
    store: Hstore,
}

#[fixture]
fn db_transaction() -> PgConnection {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL to be defined (may use .env)");

    let mut conn = PgConnection::establish(&database_url).unwrap();

    conn.batch_execute(
        r#"
        CREATE EXTENSION IF NOT EXISTS hstore;
        DROP TABLE IF EXISTS hstore_table;
        CREATE TABLE hstore_table (
          id SERIAL PRIMARY KEY,
          store hstore NOT NULL
        );"#,
    )
    .unwrap();
    conn.begin_test_transaction().unwrap();
    conn.batch_execute(
        r#"
        INSERT INTO hstore_table (id, store)
        VALUES (1, 'a=>1,b=>2'::hstore);"#,
    )
    .unwrap();

    conn
}

#[rstest]
fn metadata(mut db_transaction: PgConnection) {
    let mut m = Hstore::new();
    m.insert("Hello".into(), "There".into());
    m.insert("Again".into(), "Stuff".into());

    let another = HasHstore { id: 2, store: m };

    diesel::insert_into(hstore_table::table)
        .values(&another)
        .execute(&mut db_transaction)
        .expect("To insert data");

    let data: Vec<HasHstore> = hstore_table::table
        .get_results(&mut db_transaction)
        .expect("To get data");

    assert_eq!(data[0].store["a"], "1".to_string());
    assert_eq!(data[0].store["b"], "2".to_string());

    assert_eq!(data[1].store["Hello"], "There".to_string());
    assert_eq!(data[1].store["Again"], "Stuff".to_string());
}

#[rstest]
fn update(mut db_transaction: PgConnection) {
    use hstore_table::dsl::store;

    let new_store: Hstore = [("c", "3"), ("d", "4")]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    let updated: Vec<HasHstore> = diesel::update(hstore_table::table)
        .set(store.eq(new_store))
        .get_results(&mut db_transaction)
        .unwrap();

    assert_eq!(updated[0].store.contains_key("a"), false);
    assert_eq!(updated[0].store.contains_key("b"), false);
    assert_eq!(updated[0].store["c"], "3".to_string());
    assert_eq!(updated[0].store["d"], "4".to_string());
}
