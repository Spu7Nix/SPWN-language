use crate::{
    lexer::Token,
    parser::{ASTData, ExprKey, Expression, Statement, Statements, StmtKey},
    value::Value,
};

pub type InstrNum = u16;

#[derive(Default)]
pub struct Code {
    pub constants: Vec<Value>,
    pub var_names: Vec<String>,
    pub destinations: Vec<usize>,

    pub instructions: Vec<Vec<Instruction>>,
}
impl Code {
    fn add_const(&mut self, c: Value) -> InstrNum {
        (match self.constants.iter().position(|v| v == &c) {
            Some(i) => i,
            None => {
                self.constants.push(c);
                self.constants.len() - 1
            }
        }) as InstrNum
    }
    fn add_var_name(&mut self, n: String) -> InstrNum {
        (match self.var_names.iter().position(|v| v == &n) {
            Some(i) => i,
            None => {
                self.var_names.push(n);
                self.var_names.len() - 1
            }
        }) as InstrNum
    }
    fn add_destination(&mut self, n: usize) -> InstrNum {
        (match self.destinations.iter().position(|v| v == &n) {
            Some(i) => i,
            None => {
                self.destinations.push(n);
                self.destinations.len() - 1
            }
        }) as InstrNum
    }

    pub fn debug(&self) {
        println!("-------- constants --------");
        println!("{:?}", self.constants);
        println!("-------- var names --------");
        println!("{:?}", self.var_names);
        println!("--------   dests   --------");
        println!("{:?}\n", self.destinations);

        for (i, instrs) in self.instructions.iter().enumerate() {
            println!("============================> Func {}", i);
            for (i, instr) in instrs.iter().enumerate() {
                use ansi_term::Color::Yellow;
                let instr_len = 20;

                let col = Yellow.bold();

                let instr_str = format!("{:?}", instr);
                let instr_str =
                    instr_str.clone() + &String::from(" ").repeat(instr_len - instr_str.len());

                let mut s = format!("{}\t{}", i, instr_str);

                match instr {
                    Instruction::LoadConst(idx) => {
                        s += &col
                            .paint(format!("{:?}", self.constants[*idx as usize]))
                            .to_string()
                    }
                    Instruction::LoadVar(idx) => {
                        s += &col
                            .paint(format!("{:?}", self.var_names[*idx as usize]))
                            .to_string()
                    }
                    Instruction::LetVar(idx) => {
                        s += &col
                            .paint(format!("{:?}", self.var_names[*idx as usize]))
                            .to_string()
                    }
                    Instruction::Jump(idx) => {
                        s += &col
                            .paint(format!("{:?}", self.destinations[*idx as usize]))
                            .to_string()
                    }
                    Instruction::JumpIfFalse(idx) => {
                        s += &col
                            .paint(format!("{:?}", self.destinations[*idx as usize]))
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
    LetVar(InstrNum),

    BuildArray(InstrNum),

    PushEmpty,
    PopTop,

    Jump(InstrNum),
    JumpIfFalse(InstrNum),
    // other shit soon
}

#[derive(Default)]
pub struct Compiler {
    pub ast_data: ASTData,
    pub code: Code,
}

impl Compiler {
    pub fn new(data: ASTData) -> Self {
        Self {
            ast_data: data,
            code: Code::default(),
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
                let c_id = self.code.add_const(val);
                self.push_instr(Instruction::LoadConst(c_id), func);
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
                let v_id = self.code.add_var_name(v);
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
        }
    }

    fn compile_stmt(&mut self, stmt: StmtKey, func: usize) {
        let stmt = self.ast_data.get_stmt(stmt);

        match stmt {
            Statement::Expr(expr) => {
                self.compile_expr(expr, func);
                self.push_instr(Instruction::PopTop, func);
            }
            Statement::Let(name, value) => {
                self.compile_expr(value, func);
                let v_id = self.code.add_var_name(name);
                self.push_instr(Instruction::LetVar(v_id), func);
            }
            Statement::If {
                branches,
                else_branch,
            } => {
                let mut end_jumps = vec![];

                for (cond, code) in branches {
                    self.compile_expr(cond, func);
                    let jump_idx = self.push_instr(Instruction::JumpIfFalse(0), func);

                    self.compile_stmts(code, func);
                    let j = self.push_instr(Instruction::Jump(0), func);
                    end_jumps.push(j);

                    let next_pos = self.instr_len(func);
                    let idx = self.code.add_destination(next_pos);
                    self.set_instr(Instruction::JumpIfFalse(idx), func, jump_idx);
                }

                let next_pos = self.instr_len(func);
                for i in end_jumps {
                    let idx = self.code.add_destination(next_pos);
                    self.set_instr(Instruction::Jump(idx), func, i);
                }
                if let Some(code) = else_branch {
                    self.compile_stmts(code, func);
                }
            }
            Statement::While { cond, code } => {
                let cond_pos = self.instr_len(func);
                self.compile_expr(cond, func);
                let jump_pos = self.push_instr(Instruction::JumpIfFalse(0), func);
                self.compile_stmts(code, func);

                let idx = self.code.add_destination(cond_pos);
                self.push_instr(Instruction::Jump(idx), func);

                let next_pos = self.instr_len(func);
                let idx = self.code.add_destination(next_pos);
                self.set_instr(Instruction::JumpIfFalse(idx), func, jump_pos);
            }
            Statement::For {
                var,
                iterator,
                code,
            } => todo!(),
        }
    }

    pub fn compile_stmts(&mut self, stmts: Statements, func: usize) {
        for i in stmts {
            self.compile_stmt(i, func);
        }
    }
}
