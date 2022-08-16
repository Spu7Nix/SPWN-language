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
        let data = unsafe { DATA.as_ref().unwrap() };

        match data.stmts[$e[0]].0 {
            Statement::Expr(ekey) => {
                let e = &data.exprs[ekey].0;
                assert_eq!(e.deep_eq_expr(&data.exprs, &$e2.0), true);
            }
            _ => panic!(),
        };
    };
}

#[macro_export]
macro_rules! expr {
    ($expr:expr) => {
        (Expression::from($expr), crate::sources::CodeSpan::default())
    };
}

#[macro_export]
macro_rules! expr_key {
    ($expr:expr) => {{
        let exprs = &mut unsafe { DATA.as_mut().unwrap() }.exprs;
        exprs.insert($expr)
    }};
}

#[macro_export]
macro_rules! expr_op {
    ($left:expr,$op:expr,$right:expr) => {{
        let exprs = &mut unsafe { DATA.as_mut().unwrap() }.exprs;
        expr!(Expression::Op(
            exprs.insert($left),
            $op,
            exprs.insert($right)
        ))
    }};

    ($op:expr,$right:expr) => {{
        let exprs = &mut unsafe { DATA.as_mut().unwrap() }.exprs;
        expr!(Expression::Unary($op, exprs.insert($right)))
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
        let d: Vec<(String, _)> = vec![
            $(
                ($k.into(), expr_dict!(@$($v)?))
            ),*
        ];

        expr!(Expression::Dict(d))
    }};

    // { k: v }
    (@$v:expr) => {{
        let exprs = &mut unsafe { DATA.as_mut().unwrap() }.exprs;
        Some(exprs.insert($v))
    }};

    // { k }
    (@) => {
        None
    }
}

#[macro_export]
macro_rules! expr_arr {
    ($($e:expr),*) => {{
        let exprs = &mut unsafe { DATA.as_mut().unwrap() }.exprs;
        expr!(Expression::Array(vec![$(exprs.insert($e)),*]))
    }}
}

fn deep_eq_expr_dict(
    // .......key.....value
    a1: &[(String, Option<ExprKey>)],
    a2: &[(String, Option<ExprKey>)],
    exprs: &ExpressionMap,
) -> bool {
    // `v` = `((String, Option<ExprKey>), (String, Option<ExprKey>))`
    for v in a1.iter().zip(a2) {
        // shallow eq check for `(<name>, None)`
        if v.0 == v.1 {
            return true;
        }

        if let Some(k1) = v.0 .1 {
            if let Some(k2) = v.1 .1 {
                return exprs[k1].0.deep_eq_expr(exprs, &exprs[k2].0) && v.0 .0 == v.1 .0;
            }
        }
    }
    false
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
                exprs[*l1].0.deep_eq_expr(exprs, &exprs[*l2].0)
                    && exprs[*r1].0.deep_eq_expr(exprs, &exprs[*r2].0)
                    && t1 == t2
            }
            (Unary(t1, o1), Unary(t2, o2)) => {
                exprs[*o1].0.deep_eq_expr(exprs, &exprs[*o2].0) && t1 == t2
            }

            (Array(a1), Array(a2)) => a1
                .iter()
                .enumerate()
                .map(|v| exprs[*v.1].0.deep_eq_expr(exprs, &exprs[a2[v.0]].0))
                .all(|a| a),
            (Dict(d1), Dict(d2)) => deep_eq_expr_dict(d1, d2, exprs),

            (Obj(o1, v1), Obj(o2, v2)) => {
                o1 == o2
                    && v1
                        .iter()
                        .enumerate()
                        .map(|v| {
                            exprs[v.1 .0].0.deep_eq_expr(exprs, &exprs[v2[v.0].0].0)
                                && exprs[v.1 .1].0.deep_eq_expr(exprs, &exprs[v2[v.0].1].0)
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
                let mut t = false;

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
                                    t = t
                                        || (exprs[k1].0.deep_eq_expr(exprs, &exprs[k2].0)
                                            && v.0 .0 == v.1 .0);
                                }
                            }

                            if let Some(k1) = v.0 .2 {
                                if let Some(k2) = v.1 .2 {
                                    t = t
                                        || (exprs[k1].0.deep_eq_expr(exprs, &exprs[k2].0)
                                            && v.0 .0 == v.1 .0);
                                }
                            }
                        }

                        t = t || exprs[*t1].0.deep_eq_expr(exprs, &exprs[*t2].0);
                    }
                }
                t
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
                    .map(|v| exprs[*v.1].0.deep_eq_expr(exprs, &exprs[a2[v.0]].0))
                    .all(|a| a)
                    && exprs[*r1].0.deep_eq_expr(exprs, &exprs[*r2].0)
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
                exprs[*c1].0.deep_eq_expr(exprs, &exprs[*c2].0)
                    && exprs[*it1].0.deep_eq_expr(exprs, &exprs[*it2].0)
                    && exprs[*if1].0.deep_eq_expr(exprs, &exprs[*if2].0)
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
                exprs[*b1].0.deep_eq_expr(exprs, &exprs[*b2].0)
                    && exprs[*i1].0.deep_eq_expr(exprs, &exprs[*i2].0)
            }

            (Member { base: b1, name: n1 }, Member { base: b2, name: n2 }) => {
                exprs[*b1].0.deep_eq_expr(exprs, &exprs[*b2].0) && n1 == n2
            }

            (TypeOf { base: b1 }, TypeOf { base: b2 }) => {
                exprs[*b1].0.deep_eq_expr(exprs, &exprs[*b2].0)
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
                    .map(|v| exprs[*v.1].0.deep_eq_expr(exprs, &exprs[p2[v.0]].0))
                    .all(|a| a)
                    && np1
                        .iter()
                        .enumerate()
                        .map(|v| {
                            v.1 .0 == np2[v.0].0
                                && exprs[v.1 .1].0.deep_eq_expr(exprs, &exprs[np2[v.0].1].0)
                        })
                        .all(|a| a)
                    && exprs[*b1].0.deep_eq_expr(exprs, &exprs[*b2].0)
            }

            (TriggerFuncCall(t1), TriggerFuncCall(t2)) => {
                exprs[*t1].0.deep_eq_expr(exprs, &exprs[*t2].0)
            }

            // easier to split these up into 2 branches
            (Maybe(Some(e1)), Maybe(Some(e2))) => exprs[*e1].0.deep_eq_expr(exprs, &exprs[*e2].0),
            (Maybe(None), Maybe(None)) => true,

            // the test for trigger functions is only `!{}` so we can ignore the statements
            // inside of the function
            (TriggerFunc(_), TriggerFunc(_)) => true,

            (Instance(e1, v1), Instance(e2, v2)) => {
                exprs[*e1].0.deep_eq_expr(exprs, &exprs[*e2].0) && deep_eq_expr_dict(v1, v2, exprs)
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
        let data = unsafe { DATA.as_ref().unwrap() };

        assert_eq!(data.stmts[$e[0]].0.deep_eq_stmt(&data.exprs, &$e2), true)
    };
}

impl DeepEq for Statement {
    fn deep_eq_stmt(&self, exprs: &ExpressionMap, b: &Statement) -> bool {
        use Statement::*;

        match (self, b) {
            (Let(n1, e1), Let(n2, e2)) | (Assign(n1, e1), Assign(n2, e2)) => {
                n1 == n2 && exprs[*e1].0.deep_eq_expr(exprs, &exprs[*e2].0)
            }

            (If { branches: b1, .. }, If { branches: b2, .. }) => b1
                .iter()
                .enumerate()
                .map(|v| exprs[v.1 .0].0.deep_eq_expr(exprs, &exprs[b2[v.0].0].0))
                .all(|a| a),

            (TryCatch { catch_var: v1, .. }, TryCatch { catch_var: v2, .. }) => v1 == v2,

            (While { cond: c1, .. }, While { cond: c2, .. }) => {
                exprs[*c1].0.deep_eq_expr(exprs, &exprs[*c2].0)
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
            ) => v1 == v2 && exprs[*i1].0.deep_eq_expr(exprs, &exprs[*i2].0),

            (Return(r1), Return(r2)) => {
                if let Some(e1) = r1 {
                    if let Some(e2) = r2 {
                        return exprs[*e1].0.deep_eq_expr(exprs, &exprs[*e2].0);
                    }
                }
                false
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
