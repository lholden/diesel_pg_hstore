use super::Hstore;
use diesel::expression::{AsExpression, Expression};
use diesel::sql_types::{Array, Text};

/// Operators on the hstore type
/// See [PostgreSQL hstore](https://www.postgresql.org/docs/current/hstore.html)
mod predicates {
    use super::Hstore;
    use diesel::pg::Pg;
    use diesel::sql_types::{Array, Bool, Text};

    type TextArray = Array<Text>;

    diesel::infix_operator!(HstoreGet, "->", Text, backend: Pg);
    diesel::infix_operator!(HstoreGetArray, "->", TextArray, backend: Pg);
    diesel::infix_operator!(HstoreConcat, "||", Hstore, backend: Pg);
    diesel::infix_operator!(HstoreHasKey, "?", Bool, backend: Pg);
    diesel::infix_operator!(HstoreHasAll, "?&", Bool, backend: Pg);
    diesel::infix_operator!(HstoreHasAny, "?|", Bool, backend: Pg);
    diesel::infix_operator!(HstoreLeftSubset, "@>", Bool, backend: Pg);
    diesel::infix_operator!(HstoreRightSubset, "<@", Bool, backend: Pg);
    diesel::infix_operator!(HstoreRemove, "-", Hstore, backend: Pg);
    diesel::prefix_operator!(HstoreFlatten, "%%", Array<Text>, backend: Pg);

    // anyelement #= hstore → anyelement
    // Replaces fields in the left operand (which must be a composite type) with matching values from hstore.
    // Not sure how to implement this

    // %# hstore → text[]
    // Converts hstore to a two-dimensional key/value array.
    // 2D arrays are not supported in diesel, this should translate to a vec of tuples
    // but it seems hard to implement in practice
    // diesel::prefix_operator!(HstoreRecords, "%#", Array<Array<Text>>, backend: Pg);
}

use self::predicates::*;

pub trait HstoreOpExtensions: Expression<SqlType = Hstore> + Sized {
    /// Returns value associated with given key, or NULL if not present.
    /// See [hstore -> text operator](https://www.postgresql.org/docs/current/hstore.html)
    fn get_value<T: AsExpression<Text>>(self, other: T) -> HstoreGet<Self, T::Expression> {
        HstoreGet::new(self, other.as_expression())
    }

    /// Returns values associated with given keys, or NULL if not present.
    /// See [hstore -> text[] operator](https://www.postgresql.org/docs/current/hstore.html)
    fn get_array<T: AsExpression<Array<Text>>>(
        self,
        other: T,
    ) -> HstoreGetArray<Self, T::Expression> {
        HstoreGetArray::new(self, other.as_expression())
    }

    /// Concatenates two hstores.
    /// See [hstore || hstore operator](https://www.postgresql.org/docs/current/hstore.html)
    fn concat<T: AsExpression<Hstore>>(self, other: T) -> HstoreConcat<Self, T::Expression> {
        HstoreConcat::new(self, other.as_expression())
    }

    /// Check whether the hstore contains a key
    /// See [hstore ? text operator](https://www.postgresql.org/docs/current/hstore.html)
    fn has_key<T: AsExpression<Text>>(self, other: T) -> HstoreHasKey<Self, T::Expression> {
        HstoreHasKey::new(self, other.as_expression())
    }

    /// Does hstore contain all the specified keys?
    /// See [hstore ?& text[] operator](https://www.postgresql.org/docs/current/hstore.html)
    fn has_all_keys<T: AsExpression<Array<Text>>>(
        self,
        other: T,
    ) -> HstoreHasAll<Self, T::Expression> {
        HstoreHasAll::new(self, other.as_expression())
    }

    /// Does hstore contain any of the specified keys?
    /// See [hstore ?| text[] operator](https://www.postgresql.org/docs/current/hstore.html)
    fn has_any_keys<T: AsExpression<Array<Text>>>(
        self,
        other: T,
    ) -> HstoreHasAny<Self, T::Expression> {
        HstoreHasAny::new(self, other.as_expression())
    }

    /// Implements Expression.contains() for Hstore
    /// Checks whether the left operand contains the right operand.
    /// See [hstore @> hstore operator](https://www.postgresql.org/docs/current/hstore.html)
    fn contains<T: AsExpression<Hstore>>(self, other: T) -> HstoreRightSubset<Self, T::Expression> {
        HstoreRightSubset::new(self, other.as_expression())
    }

    /// Implements Expression.is_contained_by() for Hstore
    /// Checks whether the left operand is contained by the right operand.
    /// See [hstore <@ hstore operator](https://www.postgresql.org/docs/current/hstore.html)
    fn is_contained_by<T: AsExpression<Hstore>>(
        self,
        other: T,
    ) -> HstoreLeftSubset<Self, T::Expression> {
        HstoreLeftSubset::new(self, other.as_expression())
    }

    // There should be a way to merge these into a single generic remove()
    // but my type-fu is too weak
    /// Remove a single key from the hstore
    /// See [hstore - text operator](https://www.postgresql.org/docs/current/hstore.html)
    fn remove_key<T: AsExpression<Text>>(self, other: T) -> HstoreRemove<Self, T::Expression> {
        HstoreRemove::new(self, other.as_expression())
    }

    /// Remove the keys in the rhs array from the hstore.
    /// See [hstore - text[] operator](https://www.postgresql.org/docs/current/hstore.html)
    fn remove_keys<T: AsExpression<Array<Text>>>(
        self,
        other: T,
    ) -> HstoreRemove<Self, T::Expression> {
        HstoreRemove::new(self, other.as_expression())
    }

    /// Remove the entries in the left hstore that are present in the rhs operand.
    /// See [hstore - hstore operator](https://www.postgresql.org/docs/current/hstore.html)
    fn difference<T: AsExpression<Hstore>>(self, other: T) -> HstoreRemove<Self, T::Expression> {
        HstoreRemove::new(self, other.as_expression())
    }

    /// Converts hstore to an array of alternating keys and values.
    /// See [%% hstore operator](https://www.postgresql.org/docs/current/hstore.html)
    fn to_flat_array(self) -> HstoreFlatten<Self> {
        HstoreFlatten::new(self)
    }
}

impl<T: Expression<SqlType = Hstore>> HstoreOpExtensions for T {}
