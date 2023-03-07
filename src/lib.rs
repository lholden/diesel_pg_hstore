//! # Postgres Hstore support for Diesel
//!
//! This crate provides an Hstore type for use with Diesel and Postgres.
//!
//! ## Usage
//!
//! Add diesel_pg_hstore to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! diesel_pg_hstore = "*"
//! ```
//!
//! Bring the crate into your project. (For example, from your `lib.rs` file)
//! ```rust,ignore
//! extern diesel_pg_hstore;
//! ```
//!
//! ### Using the Hstore type with Diesel
//!
//! The type must be present in the `table!` definition for your schema. There is currently no easy
//! way to provide this without explicitly adding it to each `table!` requiring the type manually.
//!
//!
//! If you are using the `diesel print-schema` command to regenerate your schema, you might consider
//! creating a .patch file that contains the required `use diesel_pg_hstore::Hstore;` statements for
//! bringing the `Hstore` type into scope as needed.
//!
//! Using Hstore with a `table!` statement:
//!
//! ```rust
//! # #[macro_use] extern crate diesel;
//! # extern crate diesel_pg_hstore;
//! table! {
//!     use diesel::sql_types::*;
//!     use diesel_pg_hstore::Hstore;
//!
//!     my_table {
//!         id -> Integer,
//!         some_other_column -> Text,
//!         an_hstore -> Hstore,
//!     }
//! }
//! # fn main() {}
//! ```
//!
//! ### Using the Hstore type in your code
//!
//! ```rust
//! #[macro_use] extern crate diesel;
//! extern crate diesel_pg_hstore;
//!
//! use std::collections::HashMap;
//! use diesel::prelude::*;
//! use diesel_pg_hstore::Hstore;
//!
//! table! {
//!     use diesel::sql_types::*;
//!     use diesel_pg_hstore::Hstore;
//!
//!     user_profile {
//!         id -> Integer,
//!         settings -> Hstore,
//!     }
//! }
//!
//! #[derive(Insertable, Debug, PartialEq)]
//! #[table_name="user_profile"]
//! struct NewUserProfile {
//!     settings: Hstore,
//! }
//!
//! fn main() {
//!     let mut settings = HashMap::new();
//!     settings.insert("Hello".to_string(), "World".to_string());
//!
//!     let profile = NewUserProfile { settings: Hstore::from_hashmap(settings) };
//! }
//! ```
//!
//! For your convenience, the Hstore type also provides proxy methods to the standard `HashMap`
//! functions.
//!
//! ```rust
//! use diesel_pg_hstore::Hstore;
//!
//! let mut things = Hstore::new();
//! things.insert("Hello".into(), "World".into());
//! ```
//!
//! ### Nullable hstore values
//!
//! Postgres hstore entries having a null value are simply ignored.

extern crate byteorder;
extern crate diesel;
extern crate fallible_iterator;
#[cfg(feature = "serde_derive")]
extern crate serde_derive;

use std::collections::hash_map::{Drain, Entry, IntoIter, Iter, IterMut, Keys, Values, ValuesMut};
use std::collections::HashMap;
use std::iter::FromIterator;
use std::ops::{Deref, DerefMut, Index};

#[cfg(feature = "serde_derive")]
use serde_derive::{Deserialize, Serialize};

use diesel::sql_types::SqlType;

/// The Hstore wrapper type.
#[derive(Debug, Clone, Default, PartialEq, Eq, SqlType)]
#[diesel(postgres_type(name = "hstore"))]
pub struct Hstore(HashMap<String, String>);

/// You can deref the Hstore into it's backing HashMap
///
/// ```rust
/// use diesel_pg_hstore::Hstore;
/// use std::collections::HashMap;
///
/// let mut settings = Hstore::new();
/// settings.insert("Hello".into(), "World".into());
/// let hashmap: &HashMap<String, String> = &*settings;
/// ```
impl Deref for Hstore {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// You can mutably deref the Hstore into it's backing HashMap
///
/// ```rust
/// use diesel_pg_hstore::Hstore;
/// use std::collections::HashMap;
///
/// let mut settings = Hstore::new();
/// settings.insert("Hello".into(), "World".into());
/// let mut hashmap: &mut HashMap<String, String> = &mut *settings;
/// ```
impl DerefMut for Hstore {
    fn deref_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.0
    }
}

impl Hstore {
    /// Create a new Hstore object
    pub fn new() -> Hstore {
        Hstore(HashMap::new())
    }

    /// Create a new Hstore from an existing hashmap
    ///
    /// ```rust
    /// use diesel_pg_hstore::Hstore;
    /// use std::collections::HashMap;
    ///
    /// let mut settings = HashMap::new();
    /// settings.insert("Hello".into(), "World".into());
    ///
    /// let settings_hstore = Hstore::from_hashmap(settings);
    /// ```
    pub fn from_hashmap(hm: HashMap<String, String>) -> Hstore {
        Hstore(hm)
    }

    /// Please see [HashMap.with_capacity](https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.with_capacity)
    pub fn with_capacity(capacity: usize) -> Hstore {
        Hstore(HashMap::with_capacity(capacity))
    }

    /// Please see [HashMap.capacity](#method.capacity-1)
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    /// Please see [HashMap.reserve](#method.reserve-1)
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional)
    }

    /// Please see [HashMap.shrink_to_fit](#method.shrink_to_fit-1)
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit()
    }

    /// Please see [HashMap.keys](#method.keys-1)
    pub fn keys(&self) -> Keys<String, String> {
        self.0.keys()
    }

    /// Please see [HashMap.values](#method.values-1)
    pub fn values(&self) -> Values<String, String> {
        self.0.values()
    }

    /// Please see [HashMap.values_mut](#method.values_mut-1)
    pub fn values_mut(&mut self) -> ValuesMut<String, String> {
        self.0.values_mut()
    }

    /// Please see [HashMap.iter](#method.iter-1)
    pub fn iter(&self) -> Iter<String, String> {
        self.0.iter()
    }

    /// Please see [HashMap.iter_mut](#method.iter_mut-1)
    pub fn iter_mut(&mut self) -> IterMut<String, String> {
        self.0.iter_mut()
    }

    /// Please see [HashMap.entry](#method.entry-1)
    pub fn entry(&mut self, key: String) -> Entry<String, String> {
        self.0.entry(key)
    }

    /// Please see [HashMap.len](#method.len-1)
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Please see [HashMap.is_empty](#method.is_empty-1)
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Please see [HashMap.drain](#method.drain-1)
    pub fn drain(&mut self) -> Drain<String, String> {
        self.0.drain()
    }

    /// Please see [HashMap.clear](#method.clear-1)
    pub fn clear(&mut self) {
        self.0.clear()
    }

    /// Please see [HashMap.get](#method.gt-1)
    pub fn get(&self, k: &str) -> Option<&String> {
        self.0.get(k)
    }

    /// Please see [HashMap.get_mut](#method.get_mut-1)
    pub fn get_mut(&mut self, k: &str) -> Option<&mut String> {
        self.0.get_mut(k)
    }

    /// Please see [HashMap.contains_key](#method.contains_key-1)
    pub fn contains_key(&self, k: &str) -> bool {
        self.0.contains_key(k)
    }

    /// Please see [HashMap.insert](#method.insert-1)
    pub fn insert(&mut self, k: String, v: String) -> Option<String> {
        self.0.insert(k, v)
    }

    /// Please see [HashMap.remove](#method.remove-1)
    pub fn remove(&mut self, k: &str) -> Option<String> {
        self.0.remove(k)
    }

    /// Please see [HashMap.retain](#method.retain-1)
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&String, &mut String) -> bool,
    {
        self.0.retain(f)
    }
}

impl IntoIterator for Hstore {
    type Item = (String, String);
    type IntoIter = IntoIter<String, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Hstore {
    type Item = (&'a String, &'a String);
    type IntoIter = Iter<'a, String, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut Hstore {
    type Item = (&'a String, &'a mut String);
    type IntoIter = IterMut<'a, String, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl FromIterator<(String, String)> for Hstore {
    fn from_iter<T>(iter: T) -> Hstore
    where
        T: IntoIterator<Item = (String, String)>,
    {
        Hstore(HashMap::from_iter(iter))
    }
}

impl<'a> Index<&'a str> for Hstore {
    type Output = String;

    #[inline]
    fn index(&self, index: &'a str) -> &Self::Output {
        self.0.get(index).expect("no entry found for key")
    }
}

impl Extend<(String, String)> for Hstore {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = (String, String)>,
    {
        self.0.extend(iter)
    }
}

mod impls {
    use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
    use diesel::deserialize::{self, FromSql};
    use diesel::expression::Expression;
    use diesel::pg::{Pg, PgValue};
    use diesel::query_builder::{AstPass, QueryFragment};
    use diesel::result::QueryResult;
    use diesel::serialize::{IsNull, Output, ToSql};
    use diesel::{AppearsOnTable, Queryable};
    use fallible_iterator::FallibleIterator;
    use std::collections::HashMap;
    use std::error::Error as StdError;
    use std::io::Write;
    use std::str;

    use super::Hstore;

    impl<QS> AppearsOnTable<QS> for Hstore {}

    impl Queryable<Hstore, Pg> for Hstore {
        type Row = Self;

        fn build(row: Self::Row) -> deserialize::Result<Self> {
            Ok(row)
        }
    }

    impl Expression for Hstore {
        type SqlType = Hstore;
    }

    impl FromSql<Hstore, Pg> for Hstore {
        fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
            let mut buf = bytes.as_bytes();
            let count = buf.read_i32::<BigEndian>()?;

            if count < 0 {
                return Err("Invalid entry count for hstore".into());
            }

            let mut entries = HstoreIterator {
                remaining: count,
                buf: buf,
            };

            let mut map = HashMap::new();

            while let Some((k, v)) = entries.next()? {
                map.insert(k.into(), v.into());
            }

            Ok(Hstore(map))
        }
    }

    impl ToSql<Hstore, Pg> for Hstore {
        fn to_sql<'b>(
            &'b self,
            out: &mut Output<'b, '_, Pg>,
        ) -> Result<IsNull, Box<dyn StdError + Send + Sync>> {
            let mut buf: Vec<u8> = Vec::new();
            buf.extend_from_slice(&[0; 4]);

            let mut count = 0;
            for (key, value) in &self.0 {
                count += 1;

                write_pascal_string(&key, &mut buf)?;
                write_pascal_string(&value, &mut buf)?;
            }

            let count = count as i32;
            (&mut buf[0..4]).write_i32::<BigEndian>(count).unwrap();

            out.write_all(&buf)?;
            Ok(IsNull::No)
        }
    }

    // Required for ExecuteDsl and LoadQuery
    impl QueryFragment<Pg> for Hstore {
        fn walk_ast<'b>(&'b self, mut pass: AstPass<'_, 'b, Pg>) -> QueryResult<()> {
            let query_entries = self
                .0
                .iter()
                .map(|(k, v)| format!("{}=>{}", k, v))
                .reduce(|acc, e| acc + "," + &e)
                .unwrap();
            pass.push_sql("'");
            pass.push_sql(query_entries.as_str());
            pass.push_sql("'::hstore");
            Ok(())
        }
    }

    fn write_pascal_string(
        s: &str,
        buf: &mut Vec<u8>,
    ) -> Result<(), Box<dyn StdError + Sync + Send>> {
        let size: i32 = s.len() as i32;
        buf.write_i32::<BigEndian>(size).unwrap();
        buf.extend_from_slice(s.as_bytes());
        Ok(())
    }

    struct HstoreIterator<'a> {
        remaining: i32,
        buf: &'a [u8],
    }

    impl<'a> HstoreIterator<'a> {
        fn consume(
            &mut self,
        ) -> Result<Option<(&'a str, Option<&'a str>)>, Box<dyn StdError + Sync + Send>> {
            if self.remaining == 0 {
                if !self.buf.is_empty() {
                    return Err("invalid buffer size".into());
                }
                return Ok(None);
            }

            self.remaining -= 1;

            let key_len = self.buf.read_i32::<BigEndian>()?;
            if key_len < 0 {
                return Err("invalid key length".into());
            }
            let (key, buf) = self.buf.split_at(key_len as usize);
            let key = str::from_utf8(key)?;
            self.buf = buf;

            let value_len = self.buf.read_i32::<BigEndian>()?;
            let value = if value_len < 0 {
                None
            } else {
                let (value, buf) = self.buf.split_at(value_len as usize);
                let value = str::from_utf8(value)?;
                self.buf = buf;
                Some(value)
            };

            Ok(Some((key, value)))
        }
    }

    impl<'a> FallibleIterator for HstoreIterator<'a> {
        type Item = (&'a str, &'a str);
        type Error = Box<dyn StdError + Sync + Send>;

        #[inline]
        fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
            while let Some(res) = self.consume()? {
                match res {
                    (key, Some(val)) => return Ok(Some((key, val))),
                    _ => continue,
                }
            }

            Ok(None)
        }

        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            let len = self.remaining as usize;
            (len, Some(len))
        }
    }
}
