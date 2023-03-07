#![feature(test)]

extern crate diesel;
extern crate diesel_pg_hstore;
extern crate dotenv;

use std::env;

use diesel::connection::SimpleConnection;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::Connection;

use diesel_pg_hstore::Hstore;

fn connection() -> PgConnection {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL to be defined (may use .env)");
    PgConnection::establish(&database_url).unwrap()
}

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

fn make_table(db: &mut PgConnection) {
    db.batch_execute(
        r#"
        CREATE EXTENSION IF NOT EXISTS hstore;
        DROP TABLE IF EXISTS hstore_table;
        CREATE TABLE hstore_table (
            id SERIAL PRIMARY KEY,
            store hstore NOT NULL
        );
        INSERT INTO hstore_table (id, store)
          VALUES (1, 'a=>1,b=>2'::hstore);
    "#,
    )
    .unwrap();
}

#[test]
fn metadata() {
    let mut db = connection();
    make_table(&mut db);

    let mut m = Hstore::new();
    m.insert("Hello".into(), "There".into());
    m.insert("Again".into(), "Stuff".into());

    let another = HasHstore { id: 2, store: m };

    diesel::insert_into(hstore_table::table)
        .values(&another)
        .execute(&mut db)
        .expect("To insert data");

    let data: Vec<HasHstore> = hstore_table::table
        .get_results(&mut db)
        .expect("To get data");

    assert_eq!(data[0].store["a"], "1".to_string());
    assert_eq!(data[0].store["b"], "2".to_string());

    assert_eq!(data[1].store["Hello"], "There".to_string());
    assert_eq!(data[1].store["Again"], "Stuff".to_string());
}
