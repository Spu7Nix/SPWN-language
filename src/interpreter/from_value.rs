use super::value::Value;

// can't return a `RuntimeError` cause can't get an `area` here
// instead return a tuple of arguments to be formatted into error at a different place that
// has an `area`
type Error = (String, &'static str);

pub trait FromValue: Clone {
    fn from_value(val: Value) -> Result<Self, Error>;
}

impl FromValue for Value {
    fn from_value(val: Value) -> Result<Self, Error> {
        Ok(val)
    }
}

impl FromValue for bool {
    fn from_value(val: Value) -> Result<Self, Error> {
        if let Value::Bool(b) = val {
            Ok(b)
        } else {
            Err((val.get_type().to_str(), "bool"))
        }
    }
}

impl FromValue for String {
    fn from_value(val: Value) -> Result<Self, Error> {
        if let Value::String(s) = val {
            Ok(s)
        } else {
            Err((val.get_type().to_str(), "string"))
        }
    }
}

// TODO: other values

macro_rules! value_to_num {
    ($($in:ty)*; $($fn:ty)*) => {
        $(
            impl FromValue for $in {
                fn from_value(val: Value) -> Result<Self, Error> {
                    if let Value::Int(n) = val {
                        if ((<$in>::MIN as isize)..(<$in>::MAX as isize)).contains(&n) {
                            Ok(n as $in)
                        } else {
                            panic!("cannot cannot cast `isize` to `{}`! (value `{}` too large for `{}`)", stringify!($n), n, stringify!($n))
                        }
                    } else {
                        Err((val.get_type().to_str(), "int"))
                    }
                }
            }
        )*
        $(
            impl FromValue for $fn {
                fn from_value(val: Value) -> Result<Self, Error> {
                    if let Value::Float(n) = val {
                        if ((<$fn>::MIN as f64)..(<$fn>::MAX as f64)).contains(&n) {
                            Ok(n as $fn)
                        } else {
                            panic!("cannot cannot cast `f64` to `{}`! (value `{}` too large for `{}`)", stringify!($n), n, stringify!($n))
                        }
                    } else {
                        Err((val.get_type().to_str(), "float"))
                    }
                }
            }
        )*
    };
}
value_to_num! { i16 i32 i64 i128 isize; f32 f64 }

// generates a load of impl's that impl the trait for differnet length tuples (up to 25)
// rust still doesnt support variable length generics so this is the only solution
pub trait FromValueList {
    fn from_value_list(values: &[Value]) -> Result<Self, Error>
    where
        Self: Sized;
}

macro_rules! tuple_value_list {

    ( $first:ident $( $name:ident )* ) => {
        tuple_value_list!( 0usize; $( $name )* );
    };

    ( $count:expr ; $first:ident $( $name:ident )* ) => {
        impl<
            $(
                $name: FromValue,
            )*
        > FromValueList for (
            $(
                $name,
            )*
        ) {
            fn from_value_list(values: &[Value]) -> Result<Self, Error>
                where Self: Sized
            {
                Ok((
                    $(
                        $name::from_value(values[$count].clone())?,
                    )*
                ))
            }
        }

        tuple_value_list!( $count + 1usize ; $( $name )* );
    };

    ( $count:expr ; ) => {};
}

tuple_value_list! { A B C D E F G H I J K L M N O P Q R S T U V W X Y Z }
