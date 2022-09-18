use crate::vm::error::RuntimeError;

use crate::vm::context::FullContext;

use crate::vm::instructions::InstrData;
use crate::vm::value::{StoredValue, Value};

use crate::run_helper;

use crate::vm::interpreter::Globals;

macro_rules! ops {
    (
        [$globals:ident]
        $(
            $name:ident($op_tok:tt) [$arg_num:literal] {
                $(
                    ( $($arg:ident: $typ:ident),+ ) -> $ret_type:ident => $code:expr,
                )+
            }
        )*
    ) => {
        #[derive(Clone, Debug, Copy)]
        pub enum Operator {
            $($name),*
        }

        impl Operator {
            pub fn to_str(&self) -> &'static str {
                match self {
                    $(Self::$name => stringify!($op_tok)),*
                }
            }
        }

        pub fn run_call_op(
            $globals: &mut Globals,
            data: &InstrData,
            context: &mut FullContext,
            operator: Operator,
        ) -> Result<(), RuntimeError> {
            run_helper!(context, $globals, data);

            let value = match operator {
                $(
                    Operator::$name => {
                        let mut args = (0..$arg_num).map(|_| pop!(Deep)).collect::<Vec<_>>();
                        args.reverse();
                        match &args.as_slice() {
                            $(
                                &[$( StoredValue {value: Value::$typ($arg), ..}  ),+] => Value::$ret_type($code),
                            )+
                            _ => {
                                // todO: custom impl check ???
                                return match args.len() {
                                    1 => Err(RuntimeError::InvalidUnaryOperand {
                                        a: args[0].clone(),
                                        op: operator,
                                        area: area!(),
                                    }),
                                    _ => Err(RuntimeError::InvalidOperands {
                                        a: args[0].clone(),
                                        b: args[1].clone(),
                                        op: operator,
                                        area: area!(),
                                    })
                                }
                            }
                        }
                    }
                )*
            };

            push!(Value: value.into_stored(area!()));

            Ok(())
        }

    };
}

ops! {
    [globals]

    Plus(+)[2] {
        (a: Int,    b: Int)   -> Int    => a + b,
        (a: Float,  b: Int)   -> Float  => a + *b as f64,
        (a: Int,    b: Float) -> Float  => *a as f64 + b,
        (a: Float,  b: Float) -> Float  => a + b,

        (a: Array, b: Array) -> Array => a.iter().chain(b.iter()).cloned().collect(),

        (a: String, b: String) -> String => a.clone() + &b,
    }
    Minus(-)[2] {
        (a: Int, b: Int) -> Int => a - b,
        (a: Float, b: Int) -> Float => a - *b as f64,
        (a: Int, b: Float) -> Float => *a as f64 - b,
        (a: Float, b: Float) -> Float => a - b,
    }
    Mult(*)[2] {
        (a: Int, b: Int) -> Int => a * b,
        (a: Float, b: Int) -> Float => a * *b as f64,
        (a: Int, b: Float) -> Float => *a as f64 * b,
        (a: Float, b: Float) -> Float => a * b,
    }
    Div(/)[2] {
        (a: Int, b: Int) -> Int => a / b,
        (a: Float, b: Int) -> Float => a / *b as f64,
        (a: Int, b: Float) -> Float => *a as f64 / b,
        (a: Float, b: Float) -> Float => a / b,
    }
    Modulo(%)[2] {
        (a: Int, b: Int) -> Int => a % b,
        (a: Float, b: Int) -> Float => a % *b as f64,
        (a: Int, b: Float) -> Float => *a as f64 % b,
        (a: Float, b: Float) -> Float => a % b,
    }
    Pow(^)[2] {
        (a: Int, b: Int) -> Int => a.pow(*b as u32),
    }
    // Eq(==)[2] {
    //     (a, b) -> Bool => a == b,

    // }
    // Eq,
    // Neq,
    // Gt,
    // Gte,
    // Lt,
    // Lte,

    Negate(-)[1] {
        (a: Int) -> Int => -a,
        (a: Float) -> Float => -a,
    }
    Not(!)[1] {
        (a: Bool) -> Bool => !a,
    }
}
