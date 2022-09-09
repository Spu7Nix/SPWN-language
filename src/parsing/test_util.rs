use super::ast::*;

pub trait DeepEq {
    fn deep_eq_expr(&self, _exprs: &ExpressionMap, _b: &Expression) -> bool {
        unimplemented!()
    }

    fn deep_eq_stmt(&self, _exprs: &ExpressionMap, _b: &Statement) -> bool {
        unimplemented!()
    }
}

///////////////////////////////////////////
/// Expressions
//////////////////////////////////////////

#[macro_export]
macro_rules! expr_eq {
    ($e:ident, $e2:expr) => {
        let expr = $e2;
        {
            let d = DATA.read().expect("error getting read lock (expr_eq)");
            let data = &*d;

            match data.stmts[$e[0]].t {
                Statement::Expr(ekey) => {
                    let e = &data.exprs[ekey];
                    assert_eq!(e.deep_eq_expr(&data.exprs, &expr), true);
                }
                _ => panic!(),
            };
        }
    };
}

#[macro_export]
macro_rules! expr {
    ($expr:expr) => {
        Expression::from($expr).span($crate::sources::CodeSpan::default())
    };
}

#[macro_export]
macro_rules! expr_key {
    ($expr:expr) => {{
        let expr = $expr;

        let mut d = DATA.write().expect("error getting write lock (expr_key)");
        let exprs = &mut d.exprs;

        exprs.insert(expr)
    }};
}

#[macro_export]
macro_rules! expr_op {
    ($left:expr,$op:expr,$right:expr) => {{
        let expr1 = $left;
        let expr2 = $right;

        let mut d = DATA
            .write()
            .expect("error getting write lock (expr_op [op])");
        let exprs = &mut d.exprs;

        expr!(Expression::Op(
            exprs.insert(expr1),
            $op,
            exprs.insert(expr2)
        ))
    }};

    ($op:expr,$right:expr) => {{
        let expr = $right;
        {
            let mut d = DATA
                .write()
                .expect("error getting write lock (expr_op [unary])");
            let exprs = &mut d.exprs;
            expr!(Expression::Unary($op, exprs.insert(expr)))
        }
    }};
}

#[macro_export]
macro_rules! expr_sym {
    ($name:literal) => {
        expr!(Expression::Var($name.into()))
    };

    (@$name:literal) => {
        expr!(Expression::Type($name.into()))
    };
}

#[macro_export]

macro_rules! expr_dict {
    ($($k:literal$(:$v:expr)?),*) => {{
        let vals = vec![$(
            ($k.into(), expr_dict!(@$($v)?))
        ),*];

        let mut d = DATA.write().expect("error getting write lock (expr_dict)");
        let exprs = &mut d.exprs;

        let mut d = vec![];

        for (k, v) in vals.into_iter() {
            d.push((k, v.map(|x| exprs.insert(x))).span($crate::sources::CodeSpan::default()));
        }

        expr!(Expression::Dict(d))
    }};

    (@$e:expr) => {
        Some($e)
    };

    (@) => { None }
}

#[macro_export]
macro_rules! expr_arr {
    ($($e:expr),*) => {{
        let es = vec![$($e),*];

        let mut d = DATA.write().expect("error getting write lock (expr_arr)");
        let exprs = &mut d.exprs;

        let e_keys = es.into_iter().map(|e| exprs.insert(e)).collect();

        expr!(Expression::Array(e_keys))
    }}
}

fn deep_eq_expr_dict(
    // .......key.....value
    a1: &[Spanned<(String, Option<ExprKey>)>],
    a2: &[Spanned<(String, Option<ExprKey>)>],
    exprs: &ExpressionMap,
) -> bool {
    if a1 == a2 {
        return true;
    }

    // `v` = `(&Spanned<(String, Option<ExprKey>)>, &Spanned<(String, Option<ExprKey>)>)`
    for v in a1.iter().zip(a2) {
        if let Some(k1) = v.0 .1 {
            if let Some(k2) = v.1 .1 {
                if !exprs[k1].deep_eq_expr(exprs, &exprs[k2]) && v.0 .0 == v.1 .0 {
                    return false;
                }
            }
        }
    }

    true
}

// deeply compares two expressions to check whether they are equal
impl DeepEq for Expression {
    fn deep_eq_expr(&self, exprs: &ExpressionMap, b: &Expression) -> bool {
        use Expression::*;

        // do shallow comparison check for variants like int, float, etc
        if self == b {
            return true;
        }

        // any other variant that has `ExprKeys` need to be checked manually
        match (self, b) {
            (Op(l1, t1, r1), Op(l2, t2, r2)) => {
                exprs[*l1].deep_eq_expr(exprs, &exprs[*l2])
                    && exprs[*r1].deep_eq_expr(exprs, &exprs[*r2])
                    && t1 == t2
            }
            (Unary(t1, o1), Unary(t2, o2)) => {
                exprs[*o1].deep_eq_expr(exprs, &exprs[*o2]) && t1 == t2
            }

            (Array(a1), Array(a2)) => a1
                .iter()
                .enumerate()
                .map(|v| exprs[*v.1].deep_eq_expr(exprs, &exprs[a2[v.0]]))
                .all(|a| a),

            (Dict(d1), Dict(d2)) => deep_eq_expr_dict(d1, d2, exprs),

            (Obj(o1, v1), Obj(o2, v2)) => {
                o1 == o2
                    && v1
                        .iter()
                        .enumerate()
                        .map(|v| {
                            exprs[v.1 .0].deep_eq_expr(exprs, &exprs[v2[v.0].0])
                                && exprs[v.1 .1].deep_eq_expr(exprs, &exprs[v2[v.0].1])
                        })
                        .all(|a| a)
            }

            (
                Macro {
                    args: a1,
                    ret_type: r1,
                    ..
                },
                Macro {
                    args: a2,
                    ret_type: r2,
                    ..
                },
            ) => {
                if a1 == a2 && r1 == r2 {
                    return true;
                }
                if let Some(t1) = r1 {
                    if let Some(t2) = r2 {
                        for v in a1.iter().zip(a2) {
                            // shallow eq check
                            if v.0 == v.1 {
                                return true;
                            }

                            if let Some(k1) = v.0 .1 {
                                if let Some(k2) = v.1 .1 {
                                    if !exprs[k1].deep_eq_expr(exprs, &exprs[k2])
                                        && v.0 .0 == v.1 .0
                                    {
                                        return false;
                                    }
                                }
                            }

                            if let Some(k1) = v.0 .2 {
                                if let Some(k2) = v.1 .2 {
                                    if !exprs[k1].deep_eq_expr(exprs, &exprs[k2])
                                        && v.0 .0 == v.1 .0
                                    {
                                        return false;
                                    }
                                }
                            }
                        }

                        if !exprs[*t1].deep_eq_expr(exprs, &exprs[*t2]) {
                            return false;
                        }
                    }
                }

                true
            }
            (
                MacroPattern {
                    args: a1,
                    ret_type: r1,
                },
                MacroPattern {
                    args: a2,
                    ret_type: r2,
                },
            ) => {
                a1.iter()
                    .enumerate()
                    .map(|v| exprs[*v.1].deep_eq_expr(exprs, &exprs[a2[v.0]]))
                    .all(|a| a)
                    && exprs[*r1].deep_eq_expr(exprs, &exprs[*r2])
            }

            (
                Ternary {
                    cond: c1,
                    if_true: it1,
                    if_false: if1,
                },
                Ternary {
                    cond: c2,
                    if_true: it2,
                    if_false: if2,
                },
            ) => {
                exprs[*c1].deep_eq_expr(exprs, &exprs[*c2])
                    && exprs[*it1].deep_eq_expr(exprs, &exprs[*it2])
                    && exprs[*if1].deep_eq_expr(exprs, &exprs[*if2])
            }

            (
                Index {
                    base: b1,
                    index: i1,
                },
                Index {
                    base: b2,
                    index: i2,
                },
            ) => {
                exprs[*b1].deep_eq_expr(exprs, &exprs[*b2])
                    && exprs[*i1].deep_eq_expr(exprs, &exprs[*i2])
            }

            (Member { base: b1, name: n1 }, Member { base: b2, name: n2 }) => {
                exprs[*b1].deep_eq_expr(exprs, &exprs[*b2]) && n1 == n2
            }

            (TypeOf { base: b1 }, TypeOf { base: b2 }) => {
                exprs[*b1].deep_eq_expr(exprs, &exprs[*b2])
            }

            (
                Call {
                    base: b1,
                    params: p1,
                    named_params: np1,
                },
                Call {
                    base: b2,
                    params: p2,
                    named_params: np2,
                },
            ) => {
                p1.iter()
                    .enumerate()
                    .map(|v| exprs[*v.1].deep_eq_expr(exprs, &exprs[p2[v.0]]))
                    .all(|a| a)
                    && np1
                        .iter()
                        .enumerate()
                        .map(|v| {
                            v.1 .0 == np2[v.0].0
                                && exprs[v.1 .1].deep_eq_expr(exprs, &exprs[np2[v.0].1])
                        })
                        .all(|a| a)
                    && exprs[*b1].deep_eq_expr(exprs, &exprs[*b2])
            }

            (TriggerFuncCall(t1), TriggerFuncCall(t2)) => {
                exprs[*t1].deep_eq_expr(exprs, &exprs[*t2])
            }

            // easier to split these up into 2 branches
            (Maybe(Some(e1)), Maybe(Some(e2))) => exprs[*e1].deep_eq_expr(exprs, &exprs[*e2]),
            (Maybe(None), Maybe(None)) => true,

            // the test for trigger functions is only `!{}` so we can ignore the statements
            // inside of the function
            (TriggerFunc(_), TriggerFunc(_)) => true,

            (Instance(e1, v1), Instance(e2, v2)) => {
                exprs[*e1].deep_eq_expr(exprs, &exprs[*e2])
                    && exprs[*v1].deep_eq_expr(exprs, &exprs[*v2])
            }

            _ => false,
        }
    }
}

impl From<i64> for Expression {
    fn from(v: i64) -> Self {
        Self::Int(v)
    }
}

impl From<f64> for Expression {
    fn from(v: f64) -> Self {
        Self::Float(v)
    }
}

impl From<bool> for Expression {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}

impl From<&str> for Expression {
    fn from(v: &str) -> Self {
        Self::String(v.into())
    }
}

///////////////////////////////////////////
/// Statements
//////////////////////////////////////////

#[macro_export]
macro_rules! stmt_eq {
    ($e:ident, $e2:expr) => {
        let expr = $e2;
        {
            let d = DATA.read().expect("error getting read lock");
            let data = &*d;

            assert_eq!(data.stmts[$e[0]].deep_eq_stmt(&data.exprs, &expr), true)
        }
    };
}

impl DeepEq for Statement {
    fn deep_eq_stmt(&self, exprs: &ExpressionMap, b: &Statement) -> bool {
        use Statement::*;

        if self == b {
            return true;
        }

        match (self, b) {
            (Let(n1, e1), Let(n2, e2)) | (Assign(n1, e1), Assign(n2, e2)) => {
                n1 == n2 && exprs[*e1].deep_eq_expr(exprs, &exprs[*e2])
            }

            (If { branches: b1, .. }, If { branches: b2, .. }) => b1
                .iter()
                .enumerate()
                .map(|v| exprs[v.1 .0].deep_eq_expr(exprs, &exprs[b2[v.0].0]))
                .all(|a| a),

            (TryCatch { catch_var: v1, .. }, TryCatch { catch_var: v2, .. }) => v1 == v2,

            (While { cond: c1, .. }, While { cond: c2, .. }) => {
                exprs[*c1].deep_eq_expr(exprs, &exprs[*c2])
            }

            (
                For {
                    var: v1,
                    iterator: i1,
                    ..
                },
                For {
                    var: v2,
                    iterator: i2,
                    ..
                },
            ) => v1 == v2 && exprs[*i1].deep_eq_expr(exprs, &exprs[*i2]),

            (Return(r1), Return(r2)) => {
                if let Some(e1) = r1 {
                    if let Some(e2) = r2 {
                        return exprs[*e1].deep_eq_expr(exprs, &exprs[*e2]);
                    }
                }
                false
            }

            (Impl(t1, m1), Impl(t2, m2)) => {
                exprs[*t1].deep_eq_expr(exprs, &exprs[*t2])
                    && exprs[*m1].deep_eq_expr(exprs, &exprs[*m2])
            }

            _ => false,
        }
    }
}

/// `&str` to `String` macro
#[macro_export]
macro_rules! s {
    ($s:literal) => {
        $s.into()
    };
}

#[macro_export]
macro_rules! spanned_vec {
    ($($es:expr),*) => {
        vec![$(
            $es.span($crate::sources::CodeSpan::default())
        ),*]
    };
}
