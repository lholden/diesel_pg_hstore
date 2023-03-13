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
#[macro_use]
extern crate diesel;
extern crate fallible_iterator;
#[cfg(feature = "serde_derive")]
extern crate serde_derive;

mod dsl;
mod functions;
mod hstore;

pub use crate::dsl::*;
pub use crate::functions::*;
pub use crate::hstore::*;
