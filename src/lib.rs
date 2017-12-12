extern crate diesel;
extern crate byteorder;
extern crate fallible_iterator;

use std::ops::{Index, Deref};
use std::collections::HashMap;
use std::collections::hash_map::*;
use std::iter::FromIterator;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Hstore(HashMap<String, Option<String>>);

impl Deref for Hstore {
    type Target = HashMap<String, Option<String>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Hstore {
    pub fn new() -> Hstore {
        Hstore(HashMap::new())
    }

    pub fn with_hashmap(hm: HashMap<String, Option<String>>) -> Hstore {
        Hstore(hm)
    }

    pub fn with_capacity(capacity: usize) -> Hstore {
        Hstore(HashMap::with_capacity(capacity))
    }

    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional)
    }

    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit()
    }

    pub fn keys(&self) -> Keys<String, Option<String>> {
        self.0.keys()
    }

    pub fn values(&self) -> Values<String, Option<String>> {
        self.0.values()
    }

    pub fn values_mut(&mut self) -> ValuesMut<String, Option<String>> {
        self.0.values_mut()
    }

    pub fn iter(&self) -> Iter<String, Option<String>> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<String, Option<String>> {
        self.0.iter_mut()
    }

    pub fn entry(&mut self, key: String) -> Entry<String, Option<String>> {
        self.0.entry(key)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn drain(&mut self) -> Drain<String, Option<String>> {
        self.0.drain()
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn get(&self, k: &str) -> Option<&Option<String>> {
        self.0.get(k)
    }

    pub fn get_mut(&mut self, k: &str) -> Option<&mut Option<String>> {
        self.0.get_mut(k)
    }

    pub fn contains_key(&self, k: &str) -> bool {
        self.0.contains_key(k)
    }

    pub fn insert(&mut self, k: String, v: Option<String>) -> Option<Option<String>> {
        self.0.insert(k, v)
    }

    pub fn remove(&mut self, k: &str) -> Option<Option<String>> {
        self.0.remove(k)
    }

    pub fn retain<F>(&mut self, f: F)
        where F: FnMut(&String, &mut Option<String>) -> bool
    {
        self.0.retain(f)
    }
}

impl IntoIterator for Hstore {
    type Item = (String, Option<String>);
    type IntoIter = IntoIter<String, Option<String>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Hstore {
    type Item = (&'a String, &'a Option<String>);
    type IntoIter = Iter<'a, String, Option<String>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut Hstore {
    type Item = (&'a String, &'a mut Option<String>);
    type IntoIter = IterMut<'a, String, Option<String>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl FromIterator<(String, Option<String>)> for Hstore {
    fn from_iter<T>(iter: T) -> Hstore
        where T: IntoIterator<Item = (String, Option<String>)>
    {
        Hstore(HashMap::from_iter(iter))
    }
}

impl<'a> Index<&'a str> for Hstore {
    type Output = Option<String>;

    #[inline]
    fn index(&self, index: &'a str) -> &Self::Output {
        self.0.get(index).expect("no entry found for key")
    }
}

impl Extend<(String, Option<String>)> for Hstore {
    fn extend<T>(&mut self, iter: T)
        where T: IntoIterator<Item = (String, Option<String>)>
    {
        self.0.extend(iter)
    }
}

mod impls {
    use std::str;
    use std::error::Error;
    use std::io::Write;
    use std::collections::HashMap;
    use fallible_iterator::FallibleIterator;
    use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian};
    use diesel::types::impls::option::UnexpectedNullError;
    use diesel::Queryable;
    use diesel::expression::AsExpression;
    use diesel::expression::bound::Bound;
    use diesel::pg::Pg;
    use diesel::row::Row;
    use diesel::types::*;

    use super::Hstore;

    impl HasSqlType<Hstore> for Pg {
        fn metadata(lookup: &Self::MetadataLookup) -> Self::TypeMetadata {
            lookup.lookup_type("hstore")
        }
    }

    impl NotNull for Hstore {}
    impl SingleValue for Hstore {}
    impl Queryable<Hstore, Pg> for Hstore {
        type Row = Self;

        fn build(row: Self::Row) -> Self {
            row
        }
    }

    impl<'a> AsExpression<Hstore> for &'a Hstore {
        type Expression = Bound<Hstore, &'a Hstore>;

        fn as_expression(self) -> Self::Expression {
            Bound::new(self)
        }
    }

    impl FromSql<Hstore, Pg> for Hstore {
        fn from_sql(bytes: Option<&[u8]>) -> Result<Self, Box<Error + Send + Sync>> {
            let mut buf = match bytes {
                Some(bytes) => bytes,
                None => return Err(Box::new(UnexpectedNullError {
                    msg: "Unexpected null for non-null column".to_string(),
                })),
            };
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
                map.insert(k.into(), v.map(|v| v.into()));
            }

            Ok(Hstore(map))
        }
    }

    impl FromSqlRow<Hstore, Pg> for Hstore {
        fn build_from_row<T: Row<Pg>>(row: &mut T) -> Result<Self, Box<Error + Send + Sync>> {
            Hstore::from_sql(row.take())
        }
    }

    impl ToSql<Hstore, Pg> for Hstore {
        fn to_sql<W>(&self, out: &mut ToSqlOutput<W, Pg>) -> Result<IsNull, Box<Error + Send + Sync>>
            where W: Write
        {
            let mut buf: Vec<u8> = Vec::new();
            buf.extend_from_slice(&[0; 4]);

            let mut count = 0;
            for (key, value) in &self.0 {
                count += 1;

                write_pascal_string(&key, &mut buf)?;

                match *value {
                    Some(ref value) => {
                        write_pascal_string(value, &mut buf)?;
                    }
                    None => buf.write_i32::<BigEndian>(-1).unwrap(),
                }
            }

            let count = count as i32;
            (&mut buf[0..4])
                .write_i32::<BigEndian>(count)
                .unwrap();

            out.write_all(&buf)?;
            Ok(IsNull::No)
        }
    }

    fn write_pascal_string(s: &str, buf: &mut Vec<u8>) -> Result<(), Box<Error + Sync + Send>> {
        let size: i32 = s.len() as i32;
        buf.write_i32::<BigEndian>(size).unwrap();
        buf.extend_from_slice(s.as_bytes());
        Ok(())
    }

    struct HstoreIterator<'a> {
        remaining: i32,
        buf: &'a [u8],
    }

    impl<'a> FallibleIterator for HstoreIterator<'a> {
        type Item = (&'a str, Option<&'a str>);
        type Error = Box<Error + Sync + Send>;

        #[inline]
        fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
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
            }
            else {
                let (value, buf) = self.buf.split_at(value_len as usize);
                let value = str::from_utf8(value)?;
                self.buf = buf;
                Some(value)
            };

            Ok(Some((key, value)))
        }

        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            let len = self.remaining as usize;
            (len, Some(len))
        }
    }
}
