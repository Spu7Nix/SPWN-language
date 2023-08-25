#[macro_export]
macro_rules! new_id_wrapper {
    ($($name:ident : $inner:ty;)*) => {
        $(
            #[derive(
                Clone,
                Copy,
                Debug,
                PartialEq,
                Eq,
                PartialOrd,
                Ord,
                Hash,
                derive_more::Display,
                derive_more::Deref,
                derive_more::DerefMut,
                serde::Serialize,
                serde::Deserialize
            )]
            pub struct $name(pub $inner);

            impl From<usize> for $name {
                fn from(value: usize) -> Self {
                    Self(value as $inner)
                }
            }
            impl From<$inner> for $name {
                fn from(value: $inner) -> Self {
                    Self(value)
                }
            }

            impl From<$name> for usize {
                fn from(value: $name) -> Self {
                    value.0 as usize
                }
            }
            impl From<$name> for $inner {
                fn from(value: $name) -> Self {
                    value.0
                }
            }
        )*
    };
}
