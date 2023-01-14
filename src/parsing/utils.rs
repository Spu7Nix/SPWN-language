macro_rules! operators {
    (
        $(
            $assoc:ident => [$($tok:ident),*];
        )*
    ) => {
        pub mod operators {

            use crate::Token;

            #[derive(Debug, Copy, Clone, PartialEq, Eq)]
            pub enum OpType {
                Left,
                Right,
                Unary,
            }

            const OP_LIST: &[(OpType, &[Token])] = &[$((OpType::$assoc, &[$(Token::$tok),*])),*];
            pub const OP_COUNT: usize = OP_LIST.len();

            pub fn next_infix(prec: usize) -> Option<usize> {
                let mut next = prec + 1;
                while next < OP_COUNT {
                    if OP_LIST[next].0 != OpType::Unary {
                        return Some(next);
                    } else {
                        next += 1;
                    }
                }
                None
            }
            pub fn is_infix_prec(op: Token, prec: usize) -> bool {
                for (i, (typ, toks)) in OP_LIST.iter().enumerate() {
                    if *typ != OpType::Unary && toks.contains(&op) && i == prec {
                        return true
                    }
                }
                return false
            }
            pub fn unary_prec(op: Token) -> Option<usize> {
                for (i, (typ, toks)) in OP_LIST.iter().enumerate() {
                    if *typ == OpType::Unary && toks.contains(&op) {
                        return Some(i)
                    }
                }
                return None
            }
            pub fn prec_type(prec: usize) -> OpType {
                OP_LIST[prec].0
            }
        }
    };
}

operators! {
    // lowest precedence
    Right => [Assign];
    Right => [PlusEq, MinusEq, MultEq, DivEq, PowEq, ModEq, BinAndEq, BinOrEq, BinNotEq, ShiftLeftEq, ShiftRightEq];
    Left => [DotDot];
    Left => [In];
    Left => [Is];
    Left => [BinOr];
    Left => [BinAnd];
    Unary => [BinNot];
    Unary => [ExclMark];
    Left => [Eq, Neq, Gt, Gte, Lt, Lte];
    Left => [ShiftLeft, ShiftRight];
    Left => [Plus, Minus];
    Unary => [Minus];
    Left => [Mult, Div, Mod];
    Right => [Pow];
    Left => [As];
    // highest precedence
}
