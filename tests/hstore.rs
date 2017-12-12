#![feature(test)]

#[macro_use]
extern crate diesel;
extern crate diesel_hstore;
extern crate dotenv;

use std::env;
use std::collections::HashMap;

use diesel::prelude::*;
use diesel::Connection;
use diesel::pg::PgConnection;
use diesel::connection::SimpleConnection;

use diesel_hstore::Hstore;

fn connection() -> PgConnection {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL to be defined (may use .env)");
    PgConnection::establish(&database_url).unwrap()
}

table! {
    use diesel::types::*;
    use diesel_hstore::Hstore;

    hstore_table {
        id -> Integer,
        store -> Hstore,
    }
}

#[derive(Insertable, Queryable, Identifiable, Debug, PartialEq)]
#[table_name = "hstore_table"]
struct HasHstore {
    id: i32,
    store: Hstore,
}

fn make_table(db: &PgConnection) {
    db.batch_execute(r#"
        CREATE EXTENSION IF NOT EXISTS hstore;
        DROP TABLE IF EXISTS hstore_table;
        CREATE TABLE hstore_table (
            id SERIAL PRIMARY KEY,
            store hstore NOT NULL
        );
        INSERT INTO hstore_table (id, store)
          VALUES (1, 'a=>1,b=>2'::hstore);
    "#).unwrap();
}

#[test]
fn metadata() {
    let db = connection();
    make_table(&db);

    let mut m = HashMap::new();
    m.insert("Hello".into(), Some("There".into()));
    m.insert("Again".into(), Some("Stuff".into()));

    let another = HasHstore {
        id: 2,
        store: Hstore::with_hashmap(m),
    };

    diesel::insert_into(hstore_table::table)
        .values(&another)
        .execute(&db)
        .expect("To insert data");

    let data: Vec<HasHstore> = hstore_table::table
        .get_results(&db)
        .expect("To get data");

    assert_eq!(data[0].store["a"], Some("1".to_string()));
    assert_eq!(data[0].store["b"], Some("2".to_string()));

    assert_eq!(data[1].store["Hello"], Some("There".to_string()));
    assert_eq!(data[1].store["Again"], Some("Stuff".to_string()));
}
