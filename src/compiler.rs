use crate::{
    lexer::Token,
    parser::{ASTData, ExprKey, Expression, Statement, Statements, StmtKey},
    value::Value,
};

pub type InstrNum = u16;

pub struct UniqueRegister<T> {
    vec: Vec<T>,
}
impl<T: PartialEq> UniqueRegister<T> {
    pub fn new() -> Self {
        Self { vec: vec![] }
    }
    pub fn add(&mut self, c: T) -> InstrNum {
        (match self.vec.iter().position(|v| v == &c) {
            Some(i) => i,
            None => {
                self.vec.push(c);
                self.vec.len() - 1
            }
        }) as InstrNum
    }
    pub fn get(&self, idx: InstrNum) -> &T {
        &self.vec[idx as usize]
    }
}

pub struct Code {
    pub constants: UniqueRegister<Value>,
    pub names: UniqueRegister<String>,
    pub destinations: UniqueRegister<usize>,
    pub name_sets: UniqueRegister<Vec<String>>,
    pub func_info: UniqueRegister<(usize, Vec<(String, bool, bool)>)>,

    pub instructions: Vec<Vec<Instruction>>,
}
impl Code {
    pub fn new() -> Self {
        Self {
            constants: UniqueRegister::new(),
            names: UniqueRegister::new(),
            destinations: UniqueRegister::new(),
            name_sets: UniqueRegister::new(),
            func_info: UniqueRegister::new(),
            instructions: vec![],
        }
    }

    pub fn debug(&self) {
        // println!("-------- constants --------");
        // println!("{:?}", self.constants.vec);
        // println!("--------   names   --------");
        // println!("{:?}", self.names.vec);
        // println!("--------   dests   --------");
        // println!("{:?}", self.destinations.vec);
        // println!("-------- name sets --------");
        // println!("{:?}\n", self.name_sets.vec);

        for (i, instrs) in self.instructions.iter().enumerate() {
            println!("============================> Func {}", i);
            for (i, instr) in instrs.iter().enumerate() {
                use ansi_term::Color::Yellow;
                let instr_len = 25;

                let col = Yellow.bold();

                let instr_str = format!("{:?}", instr);
                let instr_str =
                    instr_str.clone() + &String::from(" ").repeat(instr_len - instr_str.len());

                let mut s = format!("{}\t{}", i, instr_str);

                match instr {
                    Instruction::LoadConst(idx) => {
                        s += &col
                            .paint(format!("{:?}", self.constants.get(*idx)))
                            .to_string()
                    }
                    Instruction::LoadVar(idx) => {
                        s += &col
                            .paint(format!("var {:?}", self.names.get(*idx)))
                            .to_string()
                    }
                    Instruction::SetVar(idx, _) => {
                        s += &col
                            .paint(format!("var {:?}", self.names.get(*idx)))
                            .to_string()
                    }
                    Instruction::LoadType(idx) => {
                        s += &col.paint(format!("@{}", self.names.get(*idx))).to_string()
                    }
                    Instruction::Jump(idx) => {
                        s += &col
                            .paint(format!("to {:?}", self.destinations.get(*idx)))
                            .to_string()
                    }
                    Instruction::JumpIfFalse(idx) => {
                        s += &col
                            .paint(format!("to {:?}", self.destinations.get(*idx)))
                            .to_string()
                    }
                    Instruction::IterNext(idx) => {
                        s += &col
                            .paint(format!("to {:?}", self.destinations.get(*idx)))
                            .to_string()
                    }
                    Instruction::BuildDict(idx) => {
                        s += &col
                            .paint(format!("with {:?}", self.name_sets.get(*idx)))
                            .to_string()
                    }
                    Instruction::MakeMacro(idx) => {
                        s += &col
                            .paint(format!(
                                "Func {:?}, args: {:?}",
                                self.func_info.get(*idx).0,
                                self.func_info.get(*idx).1
                            ))
                            .to_string()
                    }
                    Instruction::Call(idx) => {
                        s += &col
                            .paint(format!("params: {:?}", self.name_sets.get(*idx)))
                            .to_string()
                    }
                    _ => (),
                }

                println!("{}", s);
            }
        }
    }
}

#[derive(Debug)]
pub enum Instruction {
    LoadConst(InstrNum),

    Plus,
    Minus,
    Mult,
    Div,
    Mod,
    Pow,

    Eq,
    NotEq,
    Greater,
    GreaterEq,
    Lesser,
    LesserEq,

    Assign,

    Negate,

    LoadVar(InstrNum),
    SetVar(InstrNum, bool),

    LoadType(InstrNum),

    BuildArray(InstrNum),

    PushEmpty,
    PopTop,

    Jump(InstrNum),
    JumpIfFalse(InstrNum),

    ToIter,
    IterNext(InstrNum),

    DeriveScope,
    PopScope,

    BuildDict(InstrNum),

    Return,
    Continue,
    Break,

    MakeMacro(InstrNum),
    PushAnyPattern,

    MakeMacroPattern(InstrNum),

    Index,
    Call(InstrNum),

    SaveContexts,
    ReviseContexts,

    MergeContexts,

    PushNone,
    WrapMaybe,
}
// impl Instruction {
//     pub fn load_const_mapped(v: Value, compiler: &mut Compiler) -> Self {
//         let idx = compiler.code.add_const(v);
//         Self::LoadConst(idx)
//     }
//     pub fn load_var_mapped(n: String, compiler: &mut Compiler) -> Self {
//         let idx = compiler.code.add_name(n);
//         Self::LoadVar(idx)
//     }
//     pub fn set_var_mapped(n: String, mutable: bool, compiler: &mut Compiler) -> Self {
//         let idx = compiler.code.add_name(n);
//         Self::SetVar(idx, mutable)
//     }
//     pub fn load_type_mapped(n: String, compiler: &mut Compiler) -> Self {
//         let idx = compiler.code.add_name(n);
//         Self::LoadType(idx)
//     }

//     pub fn jump_mapped(pos: usize, compiler: &mut Compiler) -> Self {
//         let idx = compiler.code.add_destination(pos);
//         Self::IterNext(idx)
//     }
//     pub fn jump_false_mapped(pos: usize, compiler: &mut Compiler) -> Self {
//         let idx = compiler.code.add_destination(pos);
//         Self::IterNext(idx)
//     }
//     pub fn iter_next_mapped(pos: usize, compiler: &mut Compiler) -> Self {
//         let idx = compiler.code.add_destination(pos);
//         Self::IterNext(idx)
//     }
// }

pub struct Compiler {
    pub ast_data: ASTData,
    pub code: Code,
}

impl Compiler {
    pub fn new(data: ASTData) -> Self {
        Self {
            ast_data: data,
            code: Code::new(),
        }
    }

    fn push_instr(&mut self, i: Instruction, func: usize) -> usize {
        self.code.instructions[func].push(i);
        self.code.instructions[func].len() - 1
    }
    fn set_instr(&mut self, i: Instruction, func: usize, n: usize) {
        self.code.instructions[func][n] = i;
    }
    fn instr_len(&mut self, func: usize) -> usize {
        self.code.instructions[func].len()
    }

    fn compile_expr(&mut self, expr: ExprKey, func: usize) {
        let expr = self.ast_data.get_expr(expr);

        match expr {
            Expression::Literal(l) => {
                let val = l.to_value();
                let c_id = self.code.constants.add(val);
                self.push_instr(Instruction::LoadConst(c_id), func);
            }
            Expression::Type(name) => {
                let v_id = self.code.names.add(name);
                self.push_instr(Instruction::LoadType(v_id), func);
            }
            Expression::Op(a, op, b) => {
                self.compile_expr(a, func);
                self.compile_expr(b, func);

                macro_rules! op_helper {
                    ( $($v:ident)* ) => {
                        match op {
                            $(
                                Token::$v => self.push_instr(Instruction::$v, func),
                            )*
                            _ => unreachable!(),
                        }
                    };
                }

                op_helper!(Plus Minus Mult Div Mod Pow Eq NotEq Greater GreaterEq Lesser LesserEq Assign);
            }
            Expression::Unary(op, v) => {
                self.compile_expr(v, func);

                match op {
                    Token::Minus => self.push_instr(Instruction::Negate, func),
                    _ => unreachable!(),
                };
            }
            Expression::Var(v) => {
                let v_id = self.code.names.add(v);
                self.push_instr(Instruction::LoadVar(v_id), func);
            }
            Expression::Array(v) => {
                for i in &v {
                    self.compile_expr(*i, func);
                }
                self.push_instr(Instruction::BuildArray(v.len() as InstrNum), func);
            }
            Expression::Empty => {
                self.push_instr(Instruction::PushEmpty, func);
            }
            Expression::Dict(items) => {
                let idx = self
                    .code
                    .name_sets
                    .add(items.iter().map(|(s, _)| s.clone()).collect());
                for (_, v) in items {
                    self.compile_expr(v, func);
                }
                self.push_instr(Instruction::BuildDict(idx), func);
            }
            Expression::Block(code) => {
                self.push_instr(Instruction::DeriveScope, func);
                self.compile_stmts(code, func);
                self.push_instr(Instruction::PopScope, func);
                self.push_instr(Instruction::PushEmpty, func);
            }
            Expression::Func {
                args,
                ret_type,
                code,
            } => {
                self.code.instructions.push(vec![]);
                let func_id = self.code.instructions.len() - 1;

                self.compile_expr(code, func_id);

                let idx = self.code.func_info.add((
                    func_id,
                    args.iter()
                        .map(|(s, t, d)| (s.clone(), t.is_some(), d.is_some()))
                        .collect(),
                ));

                for (_, t, d) in args {
                    if let Some(t) = t {
                        self.compile_expr(t, func);
                    }
                    if let Some(d) = d {
                        self.compile_expr(d, func);
                    }
                }

                if let Some(ret) = ret_type {
                    self.compile_expr(ret, func);
                } else {
                    self.push_instr(Instruction::PushAnyPattern, func);
                }

                self.push_instr(Instruction::MakeMacro(idx), func);
            }
            Expression::FuncPattern { args, ret_type } => {
                for i in &args {
                    self.compile_expr(*i, func);
                }
                self.compile_expr(ret_type, func);
                self.push_instr(
                    Instruction::MakeMacroPattern(args.len() as InstrNum + 1),
                    func,
                );
            }
            Expression::Ternary {
                cond,
                if_true,
                if_false,
            } => {
                self.compile_expr(cond, func);
                let jump_if_false = self.push_instr(Instruction::JumpIfFalse(0), func);
                self.compile_expr(if_true, func);
                let jump = self.push_instr(Instruction::Jump(0), func);

                let next_pos = self.instr_len(func);
                let idx = self.code.destinations.add(next_pos);
                self.set_instr(Instruction::JumpIfFalse(idx), func, jump_if_false);

                self.compile_expr(if_false, func);

                let next_pos = self.instr_len(func);
                let idx = self.code.destinations.add(next_pos);
                self.set_instr(Instruction::Jump(idx), func, jump);
            }
            Expression::Index { base, index } => {
                self.compile_expr(base, func);
                self.compile_expr(index, func);

                self.push_instr(Instruction::Index, func);
            }
            Expression::Call {
                base,
                params,
                named_params,
            } => {
                self.compile_expr(base, func);
                let idx = self.code.name_sets.add(
                    params
                        .iter()
                        .map(|_| "".into())
                        .chain(named_params.iter().map(|(s, _)| s.clone()))
                        .collect(),
                );
                for v in params {
                    self.compile_expr(v, func);
                }
                for (_, v) in named_params {
                    self.compile_expr(v, func);
                }
                self.push_instr(Instruction::Call(idx), func);
            }
            Expression::Maybe(expr) => {
                if let Some(expr) = expr {
                    self.compile_expr(expr, func);
                    self.push_instr(Instruction::WrapMaybe, func);
                } else {
                    self.push_instr(Instruction::PushNone, func);
                }
            }
        }
    }

    fn compile_stmt(&mut self, stmt: StmtKey, func: usize) {
        let is_arrow = self.ast_data.stmt_arrows[stmt];
        let stmt = self.ast_data.get_stmt(stmt);

        if is_arrow {
            self.push_instr(Instruction::SaveContexts, func);
        }

        match stmt {
            Statement::Expr(expr) => {
                self.compile_expr(expr, func);
                self.push_instr(Instruction::PopTop, func);
            }
            Statement::Let(name, value) => {
                self.compile_expr(value, func);
                let v_id = self.code.names.add(name);
                self.push_instr(Instruction::SetVar(v_id, true), func);
            }
            Statement::Assign(name, value) => {
                self.compile_expr(value, func);
                let v_id = self.code.names.add(name);
                self.push_instr(Instruction::SetVar(v_id, false), func);
            }
            Statement::If {
                branches,
                else_branch,
            } => {
                let mut end_jumps = vec![];

                for (cond, code) in branches {
                    self.compile_expr(cond, func);
                    let jump_idx = self.push_instr(Instruction::JumpIfFalse(0), func);

                    self.push_instr(Instruction::DeriveScope, func);
                    self.compile_stmts(code, func);
                    self.push_instr(Instruction::PopScope, func);

                    let j = self.push_instr(Instruction::Jump(0), func);
                    end_jumps.push(j);

                    let next_pos = self.instr_len(func);
                    let idx = self.code.destinations.add(next_pos);
                    self.set_instr(Instruction::JumpIfFalse(idx), func, jump_idx);
                }

                let next_pos = self.instr_len(func);
                for i in end_jumps {
                    let idx = self.code.destinations.add(next_pos);
                    self.set_instr(Instruction::Jump(idx), func, i);
                }
                if let Some(code) = else_branch {
                    self.push_instr(Instruction::DeriveScope, func);
                    self.compile_stmts(code, func);
                    self.push_instr(Instruction::PopScope, func);
                }
            }
            Statement::While { cond, code } => {
                let cond_pos = self.instr_len(func);
                self.compile_expr(cond, func);
                let jump_pos = self.push_instr(Instruction::JumpIfFalse(0), func);

                self.push_instr(Instruction::DeriveScope, func);
                self.compile_stmts(code, func);
                self.push_instr(Instruction::PopScope, func);

                let idx = self.code.destinations.add(cond_pos);
                self.push_instr(Instruction::Jump(idx), func);

                let next_pos = self.instr_len(func);
                let idx = self.code.destinations.add(next_pos);
                self.set_instr(Instruction::JumpIfFalse(idx), func, jump_pos);
            }
            Statement::For {
                var,
                iterator,
                code,
            } => {
                self.compile_expr(iterator, func);
                self.push_instr(Instruction::ToIter, func);
                let iter_pos = self.push_instr(Instruction::IterNext(0), func);
                let v_id = self.code.names.add(var);

                self.push_instr(Instruction::DeriveScope, func);
                self.push_instr(Instruction::SetVar(v_id, false), func);
                self.compile_stmts(code, func);
                self.push_instr(Instruction::PopScope, func);

                let idx = self.code.destinations.add(iter_pos);
                let jump_pos = self.push_instr(Instruction::Jump(idx), func);

                let idx = self.code.destinations.add(jump_pos + 1);
                self.set_instr(Instruction::IterNext(idx), func, iter_pos);
            }
            Statement::Return(val) => {
                if let Some(val) = val {
                    self.compile_expr(val, func);
                } else {
                    self.push_instr(Instruction::PushEmpty, func);
                }
                self.push_instr(Instruction::Return, func);
            }
            Statement::Break => {
                self.push_instr(Instruction::Break, func);
            }
            Statement::Continue => {
                self.push_instr(Instruction::Continue, func);
            }
        }

        if is_arrow {
            self.push_instr(Instruction::ReviseContexts, func);
        }

        self.push_instr(Instruction::MergeContexts, func);
    }

    pub fn compile_stmts(&mut self, stmts: Statements, func: usize) {
        for i in stmts {
            self.compile_stmt(i, func);
        }
    }
}
