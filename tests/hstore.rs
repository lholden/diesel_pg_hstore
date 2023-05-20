#![feature(test)]

extern crate diesel;
extern crate diesel_pg_hstore;
extern crate dotenv;
#[macro_use]
extern crate rstest;

use std::env;

use diesel::connection::SimpleConnection;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::Connection;

use diesel_pg_hstore::{Hstore, HstoreOpExtensions};

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

fn prepare_extra_rows(db_transaction: &mut PgConnection) {
    let mut m = Hstore::new();
    m.insert("c".into(), "3".into());
    m.insert("d".into(), "4".into());

    let another = HasHstore { id: 2, store: m };
    diesel::insert_into(hstore_table::table)
        .values(&another)
        .execute(db_transaction)
        .expect("To insert data");
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

#[rstest]
fn test_operator_get(mut db_transaction: PgConnection) {
    use hstore_table::dsl::{id, store};

    let item: String = hstore_table::table
        .select(store.get_value("a"))
        .filter(id.eq(1))
        .get_result(&mut db_transaction)
        .unwrap();

    assert_eq!(item, "1");

    // XXX this requires Array<Text> to be able to handle
    // NULL values, at least when using Vec<Option<String>>
    //
    // let items: Vec<String> = hstore_table::table
    //     .select(store.get_array(vec!["a", "b", "c"]))
    //     .filter(id.eq(1))
    //     .get_result(&mut db_transaction)
    //     .unwrap();

    // assert_eq!(items, vec!["1", "2"]);
}

#[rstest]
fn test_operator_concat(mut db_transaction: PgConnection) {
    use hstore_table::dsl::store;

    let mut m = Hstore::new();
    m.insert("another".into(), "value".into());

    let result: Vec<HasHstore> = diesel::update(hstore_table::table)
        .set(store.eq(store.concat(m)))
        .get_results(&mut db_transaction)
        .unwrap();

    assert_eq!(result[0].store["a"], "1".to_string());
    assert_eq!(result[0].store["b"], "2".to_string());
    assert_eq!(result[0].store["another"], "value".to_string());
}

#[rstest]
fn test_operator_contains_key(mut db_transaction: PgConnection) {
    use hstore_table::dsl::store;

    prepare_extra_rows(&mut db_transaction);

    let result: Vec<bool> = hstore_table::table
        .select(store.has_key("a"))
        .get_results(&mut db_transaction)
        .unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(result[0], true);
    assert_eq!(result[1], false);

    let result: Vec<HasHstore> = hstore_table::table
        .filter(store.has_key("a"))
        .get_results(&mut db_transaction)
        .unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].store["a"], "1".to_string());
    assert_eq!(result[0].store["b"], "2".to_string());
}

#[rstest]
fn test_operator_contains_all(mut db_transaction: PgConnection) {
    use hstore_table::dsl::store;

    prepare_extra_rows(&mut db_transaction);

    let result: Vec<bool> = hstore_table::table
        .select(store.has_all_keys(vec!["a", "b"]))
        .get_results(&mut db_transaction)
        .unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(result[0], true);
    assert_eq!(result[1], false);

    let result: Vec<bool> = hstore_table::table
        .select(store.has_all_keys(vec!["a", "c"]))
        .get_results(&mut db_transaction)
        .unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(result[0], false);
    assert_eq!(result[1], false);
}

#[rstest]
fn test_operator_contains_any(mut db_transaction: PgConnection) {
    use hstore_table::dsl::store;

    prepare_extra_rows(&mut db_transaction);

    let result: Vec<bool> = hstore_table::table
        .select(store.has_any_keys(vec!["c", "b"]))
        .get_results(&mut db_transaction)
        .unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(result[0], true);
    assert_eq!(result[1], true);

    let result: Vec<bool> = hstore_table::table
        .select(store.has_any_keys(vec!["a", "b"]))
        .get_results(&mut db_transaction)
        .unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(result[0], true);
    assert_eq!(result[1], false);
}

#[rstest]
fn test_operator_subset(mut db_transaction: PgConnection) {
    use hstore_table::dsl::store;

    prepare_extra_rows(&mut db_transaction);

    let mut other = Hstore::new();
    other.insert("a".into(), "1".into());

    let result: Vec<HasHstore> = hstore_table::table
        .filter(store.contains(&other))
        .get_results(&mut db_transaction)
        .unwrap();

    assert_eq!(result.len(), 0);

    let result: Vec<HasHstore> = hstore_table::table
        .filter(store.is_contained_by(&other))
        .get_results(&mut db_transaction)
        .unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].store["a"], "1".to_string());
    assert_eq!(result[0].store["b"], "2".to_string());
}

#[rstest]
fn test_operator_remove(mut db_transaction: PgConnection) {
    use hstore_table::dsl::store;

    let result: Vec<HasHstore> = diesel::update(hstore_table::table)
        .set(store.eq(store.remove_key("a")))
        .get_results(&mut db_transaction)
        .unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].store.contains_key("a"), false);
    assert_eq!(result[0].store.contains_key("b"), true);
}

#[rstest]
fn test_operator_remove_array(mut db_transaction: PgConnection) {
    use hstore_table::dsl::store;

    let result: Vec<HasHstore> = diesel::update(hstore_table::table)
        .set(store.eq(store.remove_keys(vec!["a", "b"])))
        .get_results(&mut db_transaction)
        .unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].store.contains_key("a"), false);
    assert_eq!(result[0].store.contains_key("b"), false);
}

#[rstest]
fn test_operator_remove_hstore(mut db_transaction: PgConnection) {
    use hstore_table::dsl::store;

    let mut other = Hstore::new();
    other.insert("a".into(), "something".into());

    let result: Vec<HasHstore> = diesel::update(hstore_table::table)
        .set(store.eq(store.difference(other)))
        .get_results(&mut db_transaction)
        .unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].store.contains_key("a"), true);
    assert_eq!(result[0].store.contains_key("b"), true);

    let mut other = Hstore::new();
    other.insert("a".into(), "1".into());

    let result: Vec<HasHstore> = diesel::update(hstore_table::table)
        .set(store.eq(store.difference(other)))
        .get_results(&mut db_transaction)
        .unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].store.contains_key("a"), false);
    assert_eq!(result[0].store.contains_key("b"), true);
}

#[rstest]
fn test_operator_flatten(mut db_transaction: PgConnection) {
    use hstore_table::dsl::store;

    let result: Vec<String> = hstore_table::table
        .select(store.to_flat_array())
        .get_result(&mut db_transaction)
        .unwrap();

    assert_eq!(result.len(), 4);
    assert_eq!(result, vec!["a", "1", "b", "2"]);
}

#[rstest]
fn test_fn_from_array(mut db_transaction: PgConnection) {
    use diesel_pg_hstore::hstore_from_array;
    use hstore_table::dsl::{id, store};

    let result: HasHstore = diesel::update(hstore_table::table)
        .set(store.eq(hstore_from_array(vec!["a", "10", "b", "20"])))
        .filter(id.eq(1))
        .get_result(&mut db_transaction)
        .unwrap();

    assert_eq!(result.store["a"], "10");
    assert_eq!(result.store["b"], "20");
}

#[rstest]
fn test_fn_from_kv_array(mut db_transaction: PgConnection) {
    use diesel_pg_hstore::hstore_from_kv_array;
    use hstore_table::dsl::{id, store};

    let result: HasHstore = diesel::update(hstore_table::table)
        .set(store.eq(hstore_from_kv_array(vec!["a", "b"], vec!["10", "20"])))
        .filter(id.eq(1))
        .get_result(&mut db_transaction)
        .unwrap();

    assert_eq!(result.store["a"], "10");
    assert_eq!(result.store["b"], "20");
}

#[rstest]
fn test_fn_from_kv(mut db_transaction: PgConnection) {
    use diesel_pg_hstore::hstore_from_kv;
    use hstore_table::dsl::{id, store};

    let result: HasHstore = diesel::update(hstore_table::table)
        .set(store.eq(hstore_from_kv("a", "10")))
        .filter(id.eq(1))
        .get_result(&mut db_transaction)
        .unwrap();

    assert_eq!(result.store.len(), 1);
    assert_eq!(result.store["a"], "10");
}

#[rstest]
fn test_fn_to_array(mut db_transaction: PgConnection) {
    use diesel_pg_hstore::hstore_to_array;
    use hstore_table::dsl::{id, store};

    let result: Vec<String> = hstore_table::table
        .select(hstore_to_array(store))
        .filter(id.eq(1))
        .get_result(&mut db_transaction)
        .unwrap();

    assert_eq!(result.len(), 4);
    assert_eq!(result, vec!["a", "1", "b", "2"]);
}

#[rstest]
fn test_fn_to_keys(mut db_transaction: PgConnection) {
    use diesel_pg_hstore::hstore_to_keys;
    use hstore_table::dsl::{id, store};

    let result: Vec<String> = hstore_table::table
        .select(hstore_to_keys(store))
        .filter(id.eq(1))
        .get_result(&mut db_transaction)
        .unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(result, vec!["a", "b"]);
}

#[rstest]
fn test_fn_slice(mut db_transaction: PgConnection) {
    use diesel_pg_hstore::hstore_slice;
    use hstore_table::dsl::{id, store};

    let result: Hstore = hstore_table::table
        .select(hstore_slice(store, vec!["b"]))
        .filter(id.eq(1))
        .get_result(&mut db_transaction)
        .unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result["b"], "2");
}

#[rstest]
fn test_fn_exist(mut db_transaction: PgConnection) {
    use diesel_pg_hstore::hstore_exist;
    use hstore_table::dsl::{id, store};

    let result: bool = hstore_table::table
        .select(hstore_exist(store, "b"))
        .filter(id.eq(1))
        .get_result(&mut db_transaction)
        .unwrap();

    assert_eq!(result, true);

    let result: bool = hstore_table::table
        .select(hstore_exist(store, "c"))
        .filter(id.eq(1))
        .get_result(&mut db_transaction)
        .unwrap();

    assert_eq!(result, false);
}

// #[rstest]
// fn test_fn_defined(mut db_transaction: PgConnection) {
//     use hstore_table::dsl::{store, id};
//     use diesel_pg_hstore::hstore_exist;

//     let result: bool = hstore_table::table
//         .select(hstore_exist(store, "b"))
//         .filter(id.eq(1))
//         .get_result(&mut db_transaction)
//         .unwrap();

//     assert_eq!(result, true);

//     let result: bool = hstore_table::table
//         .select(hstore_exist(store, "c"))
//         .filter(id.eq(1))
//         .get_result(&mut db_transaction)
//         .unwrap();

//     assert_eq!(result, false);
// }

#[rstest]
fn test_fn_delete_key(mut db_transaction: PgConnection) {
    use diesel_pg_hstore::hstore_delete_key;
    use hstore_table::dsl::{id, store};

    let result: HasHstore = diesel::update(hstore_table::table)
        .set(store.eq(hstore_delete_key(store, "a")))
        .filter(id.eq(1))
        .get_result(&mut db_transaction)
        .unwrap();

    assert_eq!(result.store.len(), 1);
    assert_eq!(result.store["b"], "2");
}

#[rstest]
fn test_fn_delete_array(mut db_transaction: PgConnection) {
    use diesel_pg_hstore::hstore_delete_array;
    use hstore_table::dsl::{id, store};

    let result: HasHstore = diesel::update(hstore_table::table)
        .set(store.eq(hstore_delete_array(store, vec!["b"])))
        .filter(id.eq(1))
        .get_result(&mut db_transaction)
        .unwrap();

    assert_eq!(result.store.len(), 1);
    assert_eq!(result.store["a"], "1");
}

#[rstest]
fn test_fn_delete_matching(mut db_transaction: PgConnection) {
    use diesel_pg_hstore::hstore_delete_matching;
    use hstore_table::dsl::{id, store};

    let mut to_remove = Hstore::new();
    to_remove.insert("b".to_string(), "2".to_string());
    to_remove.insert("c".to_string(), "3".to_string());

    let result: HasHstore = diesel::update(hstore_table::table)
        .set(store.eq(hstore_delete_matching(store, to_remove)))
        .filter(id.eq(1))
        .get_result(&mut db_transaction)
        .unwrap();

    assert_eq!(result.store.len(), 1);
    assert_eq!(result.store["a"], "1");
}
