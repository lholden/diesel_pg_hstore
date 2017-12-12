#![feature(test)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_codegen;
extern crate diesel_hstore;
extern crate dotenv;

use std::env;
use std::collections::HashMap;

use diesel::prelude::*;
use diesel::Connection;
use diesel::pg::PgConnection;
use diesel::connection::SimpleConnection;

use diesel_hstore::HstoreHashMap;

fn database_url_from_env(backend_specific_env_var: &str) -> String {
    dotenv::dotenv().ok();
    env::var(backend_specific_env_var)
        .expect("DATABASE_URL must be set in order to run tests")
}

fn connection() -> PgConnection {
    let database_url = database_url_from_env("PG_DATABASE_URL");
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
    store: HstoreHashMap,
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
        store: HstoreHashMap::with_hashmap(m),
    };

    diesel::insert(&another)
        .into(hstore_table::table)
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
