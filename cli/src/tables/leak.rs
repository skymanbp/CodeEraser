//! Reading the package into the `'static` shapes its consumers already
//! hold (`&'static [&'static str]`, `Option<&'static T>`…): the package
//! is read once per process and lives until it ends, so each value is
//! read owned and leaked — never copied per query, and no consumer
//! signature changes. A borrowed read off the package text would leak
//! nothing, but serde cannot borrow a slice, and an escaped string (the
//! `\brief` doc tags, the `"` delimiter) cannot be borrowed at all.

use serde::de::{Deserialize, DeserializeOwned, Deserializer};

/// A shape read owned and then made `'static`.
pub trait Leak: Sized {
    type Owned: DeserializeOwned;
    fn leak(owned: Self::Owned) -> Self;
}

/// The `deserialize_with` every leaked field names.
pub fn leak<'de, D: Deserializer<'de>, T: Leak>(d: D) -> Result<T, D::Error> {
    T::Owned::deserialize(d).map(T::leak)
}

impl Leak for &'static str {
    type Owned = String;
    fn leak(owned: String) -> Self {
        Box::leak(owned.into_boxed_str())
    }
}

impl<T: Leak> Leak for &'static [T] {
    type Owned = Vec<T::Owned>;
    fn leak(owned: Self::Owned) -> Self {
        Box::leak(owned.into_iter().map(T::leak).collect::<Box<[T]>>())
    }
}

impl<T: Leak + 'static> Leak for &'static T {
    type Owned = T::Owned;
    fn leak(owned: Self::Owned) -> Self {
        Box::leak(Box::new(T::leak(owned)))
    }
}

impl<T: Leak> Leak for Option<T> {
    type Owned = Option<T::Owned>;
    fn leak(owned: Self::Owned) -> Self {
        owned.map(T::leak)
    }
}

impl<A: Leak, B: Leak> Leak for (A, B) {
    type Owned = (A::Owned, B::Owned);
    fn leak((a, b): Self::Owned) -> Self {
        (A::leak(a), B::leak(b))
    }
}

impl<T: Leak> Leak for Vec<T> {
    type Owned = Vec<T::Owned>;
    fn leak(owned: Self::Owned) -> Self {
        owned.into_iter().map(T::leak).collect()
    }
}

/// Shapes already `'static` read as they are.
macro_rules! owned {
    ($($t:ty),*) => {$(
        impl $crate::tables::leak::Leak for $t {
            type Owned = $t;
            fn leak(owned: $t) -> Self {
                owned
            }
        }
    )*};
}
pub(crate) use owned;
owned!(bool, usize, i64, u64, char, String);

/// A struct read through `Leak`: its owned twin (every field the
/// field type's `Owned`, the field's own serde attributes kept) is the
/// one serde reads, then each field is leaked into the struct as its
/// readers see it. The twin lives in an anonymous const, so it has no
/// name to collide with; the struct itself derives nothing from serde
/// (a derive on `&'static` fields would demand a `'static` input).
/// The table records write the compact form — several structs, each
/// `Name { field: Type, … }`, every one and every field `pub`.
macro_rules! leaked {
    ($(#[$m:meta])* $vis:vis struct $name:ident {
        $($(#[$fm:meta])* $fvis:vis $field:ident : $ty:ty),* $(,)?
    }) => {
        $(#[$m])*
        $vis struct $name {
            $($(#[$fm])* $fvis $field: $ty,)*
        }
        const _: () = {
            use $crate::tables::leak::Leak;
            #[derive(serde::Deserialize)]
            pub struct Owned {
                $($(#[$fm])* $field: <$ty as Leak>::Owned,)*
            }
            impl Leak for $name {
                type Owned = Owned;
                fn leak(owned: Owned) -> Self {
                    $name { $($field: <$ty as Leak>::leak(owned.$field),)* }
                }
            }
        };
    };
    ($($(#[$m:meta])* $name:ident { $($(#[$fm:meta])* $field:ident : $ty:ty),* $(,)? })+) => {$(
        $crate::tables::leak::leaked! {
            $(#[$m])*
            pub struct $name { $($(#[$fm])* pub $field: $ty),* }
        }
    )+};
}
pub(crate) use leaked;
