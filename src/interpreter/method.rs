pub trait Function<Args = ()>: Send + Sync + 'static {
    type Result;

    fn invoke(&self, args: Args) -> Self::Result;
}

pub trait Method<Instance, Args = ()>: Send + Sync + 'static {
    type Result;

    fn invoke(&self, instance: &Instance, args: Args) -> Self::Result;
}

macro_rules! tuple_impls {
    ( $( $name:ident )* ) => {
        impl<Fun, Res, $($name),*> Function<($($name,)*)> for Fun
            where
                Fun: Fn($($name),*) -> Res + Send + Sync + 'static
        {
            type Result = Res;

            fn invoke(&self, args: ($($name,)*)) -> Self::Result {
                #[allow(non_snake_case)]
                let ($($name,)*) = args;
                (self)($($name,)*)
            }
        }

        impl<Fun, Res, Instance, $($name),*> Method<Instance, ($($name,)*)> for Fun
            where
                Fun: Fn(&Instance, $($name),*) -> Res + Send + Sync + 'static,
        {
            type Result = Res;

            fn invoke(&self, instance: &Instance, args: ($($name,)*)) -> Self::Result {
                #[allow(non_snake_case)]
                let ($($name,)*) = args;
                (self)(instance, $($name,)*)
            }
        }
    };
}

macro_rules! tuple_impl_all {
    ( $first:ident $( $name:ident )* ) => {
        tuple_impls!( $first $( $name )* );

        tuple_impl_all!( $( $name )* );
    };

    () => {
        tuple_impls!();
    };
}

// generate stucts that allow function type definitions up to 26 arguments
tuple_impl_all! { A B C D E F G H I J K L M N O P Q R S T U V W X Y Z }
