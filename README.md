# Postgres Hstore for Diesel
[![](https://docs.rs/diesel_pg_hstore/badge.svg)](https://docs.rs/diesel_pg_hstore) [![](https://img.shields.io/crates/v/diesel_pg_hstore.svg)](https://crates.io/crates/diesel_pg_hstore) [![](https://travis-ci.org/lholden/diesel_pg_hstore.svg?branch=master)](https://travis-ci.org/lholden/diesel_pg_hstore)

This crate provides an Hstore type for use with Diesel and Postgres.

Currently only serializing to and from hstore columns is supported. If someone would be interested
in building out support for the Postgres [hstore query syntax](https://www.postgresql.org/docs/9.0/static/hstore.html), help would be appreciated!

## Usage

Please see the [Documentation](https://docs.rs/diesel_pg_hstore/) for more details.

Add diesel_pg_hstore to your `Cargo.toml`:

```toml
[dependencies]
diesel_pg_hstore = "*"
```

Bring the crate into your project. (For example, from your `lib.rs` file)
```rust,ignore
extern diesel_pg_hstore;
```

### Using the Hstore type with Diesel

The type must be present in the `table!` definition for your schema. There is currently no easy
way to provide this without explicitly adding it to each `table!` requiring the type manually.

Once Diesel 1.0 is out of beta, Diesel will be providing the ability for both the
`diesel print-schema` command and the `infer_schema!` macro to bring external types into scope.
For now, I recommend *not* using the `infer_schema!` macro.

If you are using the `diesel print-schema` command to regenerate your schema, you might consider
creating a .patch file that contains the required `use diesel_pg_hstore::Hstore;` statements for
bringing the `Hstore` type into scope as needed.

Using Hstore with a `table!` statement:

```rust
table! {
    use diesel::types::*;
    use diesel_pg_hstore::Hstore;

    my_table {
        id -> Integer,
        some_other_column -> Text,
        an_hstore -> Hstore,
    }
}
```

### Using the Hstore type in your code

```rust
#[macro_use] extern crate diesel;
extern crate diesel_pg_hstore;

use std::collections::HashMap;
use diesel::prelude::*;
use diesel_pg_hstore::Hstore;

table! {
    use diesel::types::*;
    use diesel_pg_hstore::Hstore;

    user_profile {
        id -> Integer,
        settings -> Hstore,
    }
}

#[derive(Insertable, Debug, PartialEq)]
#[table_name="user_profile"]
struct NewUserProfile {
    settings: Hstore,
}

fn main() {
    let mut settings = HashMap::new();
    settings.insert("Hello".to_string(), "World".to_string());

    let profile = NewUserProfile { settings: Hstore::from_hashmap(settings) };
}
```

For your convenience, the Hstore type also provides proxy methods to the standard `HashMap`
functions.

## License

diesel_pg_hstore is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

## Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

Please see the [CONTRIBUTING](CONTRIBUTING.md) file for more information.
