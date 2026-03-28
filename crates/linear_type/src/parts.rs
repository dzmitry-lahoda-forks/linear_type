/// Generates `parts()` and `parts_ref()` methods for a struct to access all fields.
pub trait Parts {
    /// Tuple of owned fields returned by `parts(self)`.
    type Owned;

    /// Tuple of borrowed fields returned by `parts_ref(&self)`.
    type Ref<'a>
    where
        Self: 'a;

    /// Destructure the type into all of its fields.
    fn parts(self) -> Self::Owned;

    /// Borrow all fields at once.
    fn parts_ref(&self) -> Self::Ref<'_>;
}

/// Generates a `Parts` impl and inherent `parts()` / `parts_ref()` methods for a struct.
#[macro_export]
macro_rules! parts {
    (
        $(#[$($meta:tt)*])*
        impl $(<$($impl_generics:tt),*>)?
        $name:ident $(<$($ty_generics:tt),*>)?
        $(where $($where_clause:tt)+)?
        {
            $($field:ident : $fty:ty),+ $(,)?
        }
    ) => {
        $crate::deny_non_exhaustive!($(#[$($meta)*])*);
        impl $(<$($impl_generics),*>)? $crate::Parts for $name $(<$($ty_generics),*>)?
        $(where $($where_clause)+)?
        {
            type Owned = ($($fty),+,);

            type Ref<'__parts> = ($(&'__parts $fty),+,)
            where
                Self: '__parts;

            fn parts(self) -> Self::Owned {
                ($(self.$field),+,)
            }

            fn parts_ref(&self) -> Self::Ref<'_> {
                ($(&self.$field),+,)
            }
        }

        impl $(<$($impl_generics),*>)? $name $(<$($ty_generics),*>)?
        $(where $($where_clause)+)?
        {
            #[must_use]
            pub fn parts(self) -> <Self as $crate::Parts>::Owned {
                <Self as $crate::Parts>::parts(self)
            }

            #[must_use]
            pub fn parts_ref(&self) -> <Self as $crate::Parts>::Ref<'_> {
                <Self as $crate::Parts>::parts_ref(self)
            }
        }
    };
}
