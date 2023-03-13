/// Functions on the hstore type
/// See [PostgreSQL hstore](https://www.postgresql.org/docs/current/hstore.html)
use super::Hstore;
use diesel::sql_types::*;

// hstore ( record ) → hstore
// Constructs an hstore from a record or row.
// hstore(ROW(1,2)) → "f1"=>"1", "f2"=>"2"
// Not sure how to implement this
// sql_function!(fn hstore(row: SqlType) -> Hstore);

sql_function! {
    /// Constructs an hstore from an array, which may be either a key/value array, or a two-dimensional array.
    /// The multi-dimensional variant is not yet supported.
    /// This implements hstore(text[])
    #[sql_name = "hstore"]
    fn hstore_from_array(arr: Array<Text>) -> Hstore;
}

sql_function! {
    /// Converts the hstore to an array of alternating key/value elements.
    fn hstore_to_array(h: Hstore) -> Array<Text>;
}

// 2D array and JSON conversions not currently supported

sql_function! {
    /// Constructs an hstore from separate key and value arrays.
    /// This implements hstore(text[], text[]).
    #[sql_name = "hstore"]
    fn hstore_from_kv_array(keys: Array<Text>, values: Array<Text>) -> Hstore;
}

sql_function! {
    /// Makes a single-item hstore.
    /// This implements hstore(text, text).
    #[sql_name = "hstore"]
    fn hstore_from_kv(key: Text, value: Text) -> Hstore;
}

sql_function! {
    /// Extracts an hstore's keys as an array.
    /// This implements the akeys(hstore) -> text[] postgres function.
    /// The set variant skeys is currently unsupported.
    #[sql_name = "akeys"]
    fn hstore_to_keys(h: Hstore) -> Array<Text>
}

sql_function! {
    /// Extracts an hstore's values as an array.
    /// This implements the avals(hstore) -> text[] postgres function.
    /// The set variant svals is currently unsupported
    #[sql_name = "avals"]
    fn hstore_to_values(h: Hstore) -> Array<Text>;
}

sql_function! {
    /// Extracts a subset of an hstore containing only the specified keys.
    /// This implements the slice (hstore, text[]) -> hstore postgres function.
    #[sql_name = "slice"]
    fn hstore_slice(h: Hstore, keys: Array<Text>) -> Hstore;
}

sql_function! {
    /// Check whether the hstore contains a key
    /// This implements the exist(hstore, text) -> boolean postgres function.
    #[sql_name = "exist"]
    fn hstore_exist(h: Hstore, k: Text) -> Bool;
}

sql_function! {
    /// Does hstore contain a non-NULL value for key?
    /// This implements the defined(hstore, text) -> boolean postgres function.
    #[sql_name = "defined"]
    fn hstore_defined(h: Hstore, k: Text) -> Bool;
}

sql_function! {
    /// Deletes pairs with matching keys.
    /// This implements the delete(hstore, text) -> hstore postgres function.
    #[sql_name = "delete"]
    fn hstore_delete_key(h: Hstore, key: Text) -> Hstore;
}

sql_function! {
    /// Deletes pairs with matching keys.
    /// This implements delete(hstore, text[]) -> hstore postgres function.
    #[sql_name = "delete"]
    fn hstore_delete_array(h: Hstore, keys: Array<Text>) -> Hstore;
}

sql_function! {
    /// Deletes pairs matching those in the second argument.
    /// This implements the delete (hstore, hstore) -> hstore postgres function.
    #[sql_name = "delete"]
    fn hstore_delete_matching(h: Hstore, other: Hstore) -> Hstore;
}

// populate_record ( anyelement, hstore ) → anyelement
// Replaces fields in the left operand (which must be a composite type) with matching values from hstore.
// populate_record(ROW(1,2), 'f1=>42'::hstore) → (42,2)
// Not sure how to implement this
