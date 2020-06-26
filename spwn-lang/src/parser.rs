use crate::ast;
/*use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;*/

use std::fs;
use std::path::PathBuf;
//use std::collections::HashMap;

use logos::Lexer;
use logos::Logos;

use std::iter::Peekable;

#[derive(Logos, Debug, PartialEq, Copy, Clone)]
pub enum Token {
    //OPERATORS
    #[token("->")]
    Arrow,

    #[token("||")]
    Or,

    #[token("&&")]
    And,

    #[token("==")]
    Equal,

    #[token("!=")]
    NotEqual,

    #[token(">=")]
    MoreOrEqual,

    #[token("<=")]
    LessOrEqual,

    #[token(">")]
    MoreThan,

    #[token("<")]
    LessThan,

    #[token("*")]
    Multiply,

    #[token("^")]
    Power,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("/")]
    Divide,

    #[token("!")]
    Exclamation,

    #[token("=")]
    Assign,

    //VALUES
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Symbol,

    #[regex(r"[0-9]+(.[0-9]+)?")]
    Number,

    #[regex("\"[^\n\r\"]*\"")]
    StringLiteral,

    #[token("true")]
    True,

    #[token("false")]
    False,

    #[regex(r"[0-9?]+[gbci]")]
    ID,

    //TERMINATORS
    #[token(",")]
    Comma,

    #[token("{")]
    OpenCurlyBracket,

    #[token("}")]
    ClosingCurlyBracket,

    #[token("[")]
    OpenSquareBracket,

    #[token("]")]
    ClosingSquareBracket,

    #[token("(")]
    OpenBracket,

    #[token(")")]
    ClosingBracket,

    #[token(":")]
    Colon,

    #[token(".")]
    Period,

    #[token("..")]
    Extract,

    //KEY WORDS
    #[token("return")]
    Return,

    #[token("add")]
    Add,

    #[token("impl")]
    Implement,

    #[token("for")]
    For,

    #[token("in")]
    In,

    #[token("error")]
    ErrorStatement,

    #[token("if")]
    If,

    #[token("else")]
    Else,

    #[token("obj")]
    Object,

    #[token("import")]
    Import,

    //STATEMENT SEPARATOR
    #[regex(r"[\n\r;]+")]
    StatementSeparator,

    #[error]
    #[regex(r"[ \t\f]+", logos::skip)]
    Error,
}

pub struct ParseNotes {
    pub closed_groups: Vec<u16>,
    pub closed_colors: Vec<u16>,
    pub closed_blocks: Vec<u16>,
    pub closed_items: Vec<u16>,
}

pub struct Tokens<'a> {
    iter: Lexer<'a, Token>,
    stack: Vec<(Token, String, core::ops::Range<usize>)>,
    //index 0 = element of iter / last element in stack
    index: usize,
}

impl<'a> Tokens<'a> {
    fn new(iter: Lexer<'a, Token>) -> Self {
        Tokens {
            iter,
            stack: Vec::new(),
            index: 0,
        }
    }
    fn next(&mut self, dont_skip_ss: bool) -> Option<Token> {
        if self.index == 0 {
            self.index = 0;
            //self.stack.shift();
            let mut next_element = self.iter.next();

            if let Some(e) = next_element {
                let slice = self.iter.slice().to_string();
                let range = self.iter.span();
                self.stack.push((e, slice, range));
            }

            if !dont_skip_ss && next_element == Some(Token::StatementSeparator) {
                next_element = self.iter.next();

                if let Some(e) = next_element {
                    let slice = self.iter.slice().to_string();
                    let range = self.iter.span();
                    self.stack.push((e, slice, range));
                }
            }
            next_element
        } else {
            self.index -= 1;
            if !dont_skip_ss
                && self.stack[self.stack.len() - self.index - 1].0 == Token::StatementSeparator
            {
                self.next(false)
            } else {
                Some(self.stack[self.stack.len() - self.index - 1].0)
            }
        }
    }
    fn previous(&mut self) -> Option<Token> {
        self.index += 1;
        let len = self.stack.len();
        if len > self.index {
            if self.stack[len - self.index - 1].0 == Token::StatementSeparator {
                self.index += 1;
            }
            Some(self.stack[len - self.index - 1].0)
        } else {
            None
        }
    }

    fn current(&self) -> Option<Token> {
        let len = self.stack.len();
        if len == 0 {
            None
        } else {
            Some(self.stack[len - self.index - 1].0)
        }
    }

    fn slice(&self) -> String {
        self.stack[self.stack.len() - self.index - 1].1.clone()
    }

    fn span(&self) -> core::ops::Range<usize> {
        self.stack[self.stack.len() - self.index - 1].2.clone()
    }
}

//type TokenList = Peekable<Lexer<Token>>;

pub fn parse_spwn(path: &PathBuf) -> Vec<ast::Statement> {
    let unparsed = fs::read_to_string(path).expect("Something went wrong reading the file");
    let tokens_iter = Token::lexer(&unparsed);

    let token_list: Vec<Token> = tokens_iter.clone().collect();
    println!("{:?}", token_list);

    let mut tokens = Tokens::new(tokens_iter);
    let mut statements = Vec::<ast::Statement>::new();

    loop {
        tokens.next(false);
        match tokens.next(false) {
            Some(_) => {
                tokens.previous();
                tokens.previous();
                statements.push(parse_statement(&mut tokens))
            }
            None => break,
        }

        println!(
            "\n{:?}\ncurrent: {:?}, {:?}",
            statements.last(),
            tokens.current(),
            tokens.slice()
        );

        match tokens.next(true) {
            Some(Token::StatementSeparator) => {}
            Some(a) => panic!(
                "Expected statement separator, found {:?}: {:?}",
                a,
                tokens.slice()
            ),
            None => break,
        }
    }

    statements
}

fn parse_cmp_stmt(tokens: &mut Tokens) -> Vec<ast::Statement> {
    let mut statements = Vec::<ast::Statement>::new();
    loop {
        tokens.next(false);
        match tokens.next(false) {
            Some(Token::ClosingCurlyBracket) | None => break,
            _ => {
                tokens.previous();
                tokens.previous();
                println!("statement done");

                statements.push(parse_statement(tokens))
            }
        }

        match tokens.next(true) {
            Some(Token::StatementSeparator) => {}
            Some(Token::ClosingCurlyBracket) => break,
            a => panic!(
                "Expected statement separator, found {:?}: {:?}",
                a,
                tokens.slice()
            ),
        }
    }
    //tokens.next(false);
    statements
}

pub fn parse_statement(tokens: &mut Tokens) -> ast::Statement {
    match tokens.next(false) {
        Some(Token::Arrow) => {
            tokens.previous();
            //parse async statement
            let mut rest_of_statement = parse_statement(tokens);
            if rest_of_statement.arrow {
                //double arrow (throw error)
            }
            rest_of_statement.arrow = true;
            rest_of_statement
        }

        Some(Token::Return) => {
            //parse return statement
            ast::Statement {
                body: ast::StatementBody::Return(parse_expr(tokens)),
                arrow: false,
                line: (0, 0),
            }
        }

        Some(Token::If) => {
            //parse if statement

            let condition = parse_expr(tokens);
            let if_body = parse_cmp_stmt(tokens);
            let else_body = match tokens.next(false) {
                Some(Token::Else) => Some(parse_cmp_stmt(tokens)),
                _ => {
                    tokens.previous();
                    None
                }
            };

            let if_statement = ast::If {
                condition,
                if_body,
                else_body,
            };

            ast::Statement {
                body: ast::StatementBody::If(if_statement),
                arrow: false,
                line: (0, 0),
            }
        }

        Some(Token::For) => {
            //parse for statement

            let symbol = match tokens.next(false) {
                Some(Token::Symbol) => tokens.slice(),
                None => panic!("Expected symbol"),
                _ => unreachable!(),
            };

            match tokens.next(false) {
                Some(Token::In) => {}
                _ => panic!("Expected keyword 'in'"),
            };

            let array = parse_expr(tokens);
            let body = parse_cmp_stmt(tokens);

            ast::Statement {
                body: ast::StatementBody::For(ast::For {
                    symbol,
                    array,
                    body,
                }),
                arrow: false,
                line: (0, 0),
            }
        }

        Some(Token::Add) => {
            //parse add object statement
            ast::Statement {
                body: ast::StatementBody::Add(parse_expr(tokens)),
                arrow: false,
                line: (0, 0),
            }
        }

        Some(Token::ErrorStatement) => {
            //parse error statement
            ast::Statement {
                body: ast::StatementBody::Error(ast::Error {
                    message: parse_expr(tokens),
                }),
                arrow: false,
                line: (0, 0),
            }
        }

        Some(Token::Implement) => {
            //parse impl statement
            ast::Statement {
                body: ast::StatementBody::Impl(ast::Implementation {
                    symbol: parse_variable(tokens),
                    members: parse_dict(tokens),
                }),
                arrow: false,
                line: (0, 0),
            }
        }

        Some(_) => {
            //either expression, call or definition, FIGURE OUT
            //parse it

            if tokens.next(false) == Some(Token::Assign) {
                //Def
                tokens.previous();

                println!("found def, current val: {:?}", tokens.current());

                let symbol = tokens.slice();

                tokens.next(false);

                let value = parse_expr(tokens);
                //tokens.next(false);
                ast::Statement {
                    body: ast::StatementBody::Definition(ast::Definition { symbol, value }),
                    arrow: false,
                    line: (0, 0),
                }
            } else {
                //expression or call
                tokens.previous();
                tokens.previous();
                let expr = parse_expr(tokens);
                if tokens.next(false) == Some(Token::Exclamation) {
                    //call
                    println!("found call");
                    ast::Statement {
                        body: ast::StatementBody::Call(ast::Call {
                            function: expr.values[0].clone(),
                        }),
                        arrow: false,
                        line: (0, 0),
                    }
                } else {
                    //expression statement
                    println!("found expr");
                    tokens.previous();
                    ast::Statement {
                        body: ast::StatementBody::Expr(expr),
                        arrow: false,
                        line: (0, 0),
                    }
                }
            }
        }

        None => {
            //end of input
            unimplemented!()
        }
    }
}

fn parse_expr(tokens: &mut Tokens) -> ast::Expression {
    let mut values = Vec::<ast::Variable>::new();
    let mut operators = Vec::<ast::Operator>::new();

    values.push(parse_variable(tokens));
    loop {
        let op = match tokens.next(false) {
            Some(t) => match parse_operator(&t) {
                Some(o) => o,
                None => break,
            },
            None => break,
        };

        operators.push(op);
        values.push(parse_variable(tokens));
    }
    tokens.previous();

    ast::Expression { values, operators }
}

fn parse_operator(token: &Token) -> Option<ast::Operator> {
    match token {
        Token::Arrow => Some(ast::Operator::Arrow),
        Token::Or => Some(ast::Operator::Or),
        Token::And => Some(ast::Operator::And),
        Token::Equal => Some(ast::Operator::Equal),
        Token::NotEqual => Some(ast::Operator::NotEqual),
        Token::MoreOrEqual => Some(ast::Operator::MoreOrEqual),
        Token::LessOrEqual => Some(ast::Operator::LessOrEqual),
        Token::LessThan => Some(ast::Operator::Less),
        Token::MoreThan => Some(ast::Operator::More),
        Token::Multiply => Some(ast::Operator::Multiply),
        Token::Power => Some(ast::Operator::Power),
        Token::Plus => Some(ast::Operator::Plus),
        Token::Minus => Some(ast::Operator::Minus),
        Token::Divide => Some(ast::Operator::Divide),
        _ => None,
    }
}

fn parse_dict(tokens: &mut Tokens) -> Vec<ast::DictDef> {
    Vec::new()
}

fn parse_args(tokens: &mut Tokens) -> Vec<ast::Argument> {
    let mut args = Vec::<ast::Argument>::new();
    loop {
        if tokens.next(false) == Some(Token::ClosingBracket) {
            tokens.next(false);
            break;
        };
        args.push(match tokens.next(false) {
            Some(Token::Assign) => {
                println!("assign ");
                tokens.previous();
                let symbol = Some(tokens.slice());
                tokens.next(false);
                let value = parse_expr(tokens);
                //tokens.previous();

                ast::Argument { symbol, value }
            }

            Some(_) => {
                tokens.previous();
                tokens.previous();
                println!("arg with no val");
                let value = parse_expr(tokens);

                ast::Argument {
                    symbol: None,
                    value,
                }
            }
            None => panic!("file ended while parsing arguments"),
        });

        match tokens.next(false) {
            Some(Token::Comma) => println!("comma"),
            Some(Token::ClosingBracket) => break,

            Some(a) => panic!(
                "Expected either comma or closing bracket, found {:?}: {}",
                a,
                tokens.slice()
            ),

            None => panic!("file ended while parsing arguments"),
        }
    }
    tokens.previous();

    args
}

fn parse_arg_def(tokens: &mut Tokens) -> Vec<(String, Option<ast::Expression>)> {
    let mut args = Vec::<(String, Option<ast::Expression>)>::new();
    loop {
        if tokens.next(false) == Some(Token::ClosingBracket) {
            println!("a ");
            break;
        };
        args.push(match tokens.next(false) {
            Some(Token::Colon) => {
                println!("assign ");
                tokens.previous();
                let symbol = tokens.slice();
                tokens.next(false);
                let value = Some(parse_expr(tokens));
                //tokens.previous();

                (symbol, value)
            }

            Some(_) => {
                tokens.previous();

                println!("arg with no val");
                (tokens.slice(), None)
            }
            None => panic!("file ended while parsing arguments"),
        });

        match tokens.next(false) {
            Some(Token::Comma) => println!("comma"),
            Some(Token::ClosingBracket) => break,

            Some(a) => panic!(
                "Expected either comma or closing bracket, found {:?}: {}",
                a,
                tokens.slice()
            ),

            None => panic!("file ended while parsing arguments"),
        }
    }
    //tokens.previous();

    args
}

fn parse_variable(tokens: &mut Tokens) -> ast::Variable {
    let mut first_token = tokens.next(false);
    println!("first token in var: {:?}", first_token);
    let operator = match first_token {
        Some(Token::Minus) => {
            first_token = tokens.next(false);
            Some(ast::UnaryOperator::Minus)
        }
        Some(Token::Exclamation) => {
            first_token = tokens.next(false);
            Some(ast::UnaryOperator::Not)
        }
        _ => None,
    };

    let value = match first_token {
        Some(Token::Number) => {
            ast::ValueLiteral::Number(tokens.slice().parse().expect("invalid number"))
        }
        Some(Token::StringLiteral) => ast::ValueLiteral::Str(ast::str_content(tokens.slice())),
        Some(Token::ID) => {
            let mut text = tokens.slice();
            let class_name = match text.pop().unwrap() {
                'g' => ast::IDClass::Group,
                'c' => ast::IDClass::Color,
                'i' => ast::IDClass::Item,
                'b' => ast::IDClass::Block,
                _ => unreachable!(),
            };

            let (unspecified, number) = match text.as_ref() {
                "?" => (true, 0),
                _ => (false, text.parse().expect("invalid number")),
            };
            ast::ValueLiteral::ID(ast::ID {
                class_name,
                unspecified,
                number,
            })
        }
        Some(Token::True) => ast::ValueLiteral::Bool(true),
        Some(Token::False) => ast::ValueLiteral::Bool(false),
        Some(Token::Symbol) => ast::ValueLiteral::Symbol(tokens.slice()),

        Some(Token::OpenSquareBracket) => {
            //Array
            let mut arr = Vec::new();

            loop {
                arr.push(parse_expr(tokens));
                match tokens.next(false) {
                    Some(Token::Comma) => {
                        //accounting for trailing comma
                        if let Some(Token::ClosingSquareBracket) = tokens.next(false) {
                            break;
                        } else {
                            tokens.previous();
                        }
                    }
                    Some(Token::ClosingSquareBracket) => break,
                    a => panic!(
                        "Expected either comma or closing square bracket, found {:?}",
                        a
                    ),
                }
            }

            ast::ValueLiteral::Array(arr)
        }

        Some(Token::Import) => ast::ValueLiteral::Import(match tokens.next(false) {
            Some(Token::StringLiteral) => PathBuf::from(ast::str_content(tokens.slice())),
            _ => panic!("Expected literal string in import"),
        }),

        Some(Token::OpenBracket) => {
            //Either enclosed expression or macro definition
            let parse_closed_expr = |tokens: &mut Tokens| {
                //expr
                let expr = ast::ValueLiteral::Expression(parse_expr(tokens));
                //consume closing bracket
                match tokens.next(false) {
                    Some(Token::ClosingBracket) => expr,
                    a => panic!("Expected closing bracket, found {:?}", a),
                }
            };

            let parse_macro_def = |tokens: &mut Tokens| {
                let args = parse_arg_def(tokens);

                println!("{:?}", args);

                let body = match tokens.next(false) {
                    Some(Token::OpenCurlyBracket) => parse_cmp_stmt(tokens),
                    a => panic!("Expected opening curly bracket, found {:?}", a),
                };

                ast::ValueLiteral::Macro(ast::Macro {
                    args,
                    body: ast::CompoundStatement { statements: body },
                })
            };
            //tokens.next(false);
            match tokens.next(false) {
                Some(Token::ClosingBracket) => {
                    tokens.previous();
                    //tokens.previous();

                    println!("nr1");
                    parse_macro_def(tokens)
                }

                Some(Token::Symbol) => match tokens.next(false) {
                    Some(Token::Comma) | Some(Token::Colon) => {
                        tokens.previous();
                        tokens.previous();
                        println!("nr2");
                        parse_macro_def(tokens)
                    }

                    Some(Token::ClosingBracket) => match tokens.next(false) {
                        Some(Token::OpenCurlyBracket) => {
                            tokens.previous();
                            tokens.previous();
                            tokens.previous();
                            println!("nr3");
                            parse_macro_def(tokens)
                        }
                        a => {
                            tokens.previous();
                            tokens.previous();
                            tokens.previous();
                            println!("expr reason: {:?}", a);
                            parse_closed_expr(tokens)
                        }
                    },

                    a => {
                        tokens.previous();
                        println!("expr reason: {:?}", a);
                        parse_closed_expr(tokens)
                    }
                },
                a => {
                    tokens.previous();
                    println!("expr reason: {:?}", a);
                    parse_closed_expr(tokens)
                }
            }
        }
        Some(Token::OpenCurlyBracket) => {
            //either dictionary or function
            match tokens.next(false) {
                Some(Token::Extract) => {
                    tokens.previous();
                    ast::ValueLiteral::Dictionary(parse_dict(tokens))
                }
                _ => match tokens.next(false) {
                    Some(Token::Colon) => {
                        tokens.previous();
                        tokens.previous();
                        ast::ValueLiteral::Dictionary(parse_dict(tokens))
                    }
                    _ => {
                        tokens.previous();
                        tokens.previous();
                        ast::ValueLiteral::CmpStmt(ast::CompoundStatement {
                            statements: parse_cmp_stmt(tokens),
                        })
                    }
                },
            }
        }

        _ => panic!("Not a value"),
    };

    let mut path = Vec::<ast::Path>::new();

    loop {
        match tokens.next(true) {
            Some(Token::OpenSquareBracket) => {
                let index = parse_expr(tokens);
                match tokens.next(false) {
                    Some(Token::ClosingSquareBracket) => path.push(ast::Path::Index(index)),
                    _ => panic!("Expected closing square bracket after index"),
                }
            }
            Some(Token::OpenBracket) => path.push(ast::Path::Call(parse_args(tokens))),
            Some(Token::Period) => match tokens.next(false) {
                Some(Token::Symbol) => path.push(ast::Path::Member(tokens.slice())),
                _ => panic!("Expected member name"),
            },

            _ => break,
        }
    }
    tokens.previous();

    ast::Variable {
        operator,
        value,
        path,
    }
}

//OLD PEST PARSER
/*#[derive(Parser)]
#[grammar = "spwn.pest"]
pub struct SPWNParser;

pub struct ParseNotes {
    pub closed_groups: Vec<u16>,
    pub closed_colors: Vec<u16>,
    pub closed_blocks: Vec<u16>,
    pub closed_items: Vec<u16>,
}

pub fn parse_spwn(path: &PathBuf) -> (Vec<ast::Statement>, ParseNotes) {
    let unparsed = fs::read_to_string(path).expect("Something went wrong reading the file");

    let parse_tree = SPWNParser::parse(Rule::spwn, &unparsed)
        .expect("unsuccessful parse")
        .next()
        .unwrap(); // get and unwrap the `spwn` rule; never fails

    println!("got parse tree...");
    let mut notes = ParseNotes {
        closed_groups: Vec::new(),
        closed_colors: Vec::new(),
        closed_blocks: Vec::new(),
        closed_items: Vec::new(),
    };

    let parsed = parse_statements(&mut parse_tree.into_inner(), &mut notes);
    (parsed, notes)
}

pub fn parse_statements(
    statements: &mut pest::iterators::Pairs<Rule>,
    notes: &mut ParseNotes,
) -> Vec<ast::Statement> {
    let mut stmts: Vec<ast::Statement> = vec![];

    for unpacked in statements {
        let mut inner = unpacked.clone().into_inner();
        let mut async_arrow = false;
        let first_elem = match inner.next() {
            Some(val) => val,
            None => break, //EOF
        };
        let statement = match first_elem.as_rule() {
            Rule::async_arrow => {
                async_arrow = true;
                inner.next().unwrap()
            }
            _ => first_elem,
        };
        stmts.push(ast::Statement {
            body: match statement.as_rule() {
                Rule::def => {
                    let mut inner = statement.into_inner();
                    let first = inner.next().unwrap();
                    match first.as_rule() {
                        Rule::expr => ast::StatementBody::Definition(ast::Definition {
                            symbol: "*".to_string(),
                            value: parse_expr(first, notes),
                        }),
                        Rule::symbol => {
                            let value = parse_expr(inner.next().unwrap(), notes);
                            ast::StatementBody::Definition(ast::Definition {
                                symbol: first.as_span().as_str().to_string(),
                                value,
                            })
                        }
                        _ => unreachable!(),
                    }
                }
                Rule::call => ast::StatementBody::Call(ast::Call {
                    function: parse_variable(statement.into_inner().next().unwrap(), notes),
                }),

                Rule::if_stmt => {
                    let mut inner = statement.into_inner();
                    ast::StatementBody::If(ast::If {
                        condition: parse_expr(inner.next().unwrap(), notes),
                        if_body: parse_statements(&mut inner.next().unwrap().into_inner(), notes),
                        else_body: match inner.next() {
                            Some(body) => match body.as_rule() {
                                Rule::cmp_stmt => {
                                    Some(parse_statements(&mut body.into_inner(), notes))
                                }
                                Rule::if_else => {
                                    let ret = Some(parse_statements(&mut body.into_inner(), notes));

                                    ret
                                }
                                _ => unreachable!(),
                            },
                            None => None,
                        },
                    })
                }
                Rule::add_obj => ast::StatementBody::Add(parse_expr(
                    statement.into_inner().next().unwrap(),
                    notes,
                )),

                Rule::for_loop => {
                    let mut inner = statement.into_inner();
                    ast::StatementBody::For(ast::For {
                        symbol: inner.next().unwrap().as_span().as_str().to_string(),
                        array: parse_expr(inner.next().unwrap(), notes),
                        body: parse_statements(&mut inner.next().unwrap().into_inner(), notes),
                    })
                }

                Rule::implement => {
                    let mut inner = statement.into_inner();
                    ast::StatementBody::Impl(ast::Implementation {
                        symbol: parse_variable(inner.next().unwrap(), notes),
                        members: parse_dict(inner.next().unwrap(), notes),
                    })
                }

                Rule::error => ast::StatementBody::Error(ast::Error {
                    message: parse_expr(statement.into_inner().next().unwrap(), notes),
                }),

                Rule::expr => ast::StatementBody::Expr(parse_expr(statement, notes)),
                Rule::retrn => ast::StatementBody::Return(match statement.into_inner().next() {
                    Some(expr) => parse_expr(expr, notes),

                    None => ast::Expression {
                        // null expression
                        values: vec![ast::Variable {
                            operator: None,
                            value: ast::ValueLiteral::Null,
                            path: Vec::new(),
                        }],
                        operators: Vec::new(),
                    },
                }),
                Rule::EOI => ast::StatementBody::EOI,
                _ => {
                    println!(
                        "{:?} is not added to parse_statements yet",
                        statement.as_rule()
                    );
                    ast::StatementBody::EOI
                }
            },

            arrow: async_arrow,
            line: unpacked.as_span().start_pos().line_col(),
        })
    }
    stmts
}

pub fn parse_path(pair: Pair<Rule>, notes: &mut ParseNotes) -> ast::Path {
    /*let parse_args = |arg: Pair<Rule>| {
        let mut argument = arg.into_inner();
        let first = argument.next().unwrap();
        match first.as_rule() {
            Rule::symbol => ast::Argument {
                symbol: Some(first.as_span().as_str().to_string()),
                value: parse_expr(argument.next().unwrap(), notes),
            },
            Rule::expr => ast::Argument {
                symbol: None,
                value: parse_expr(first, notes),
            },
            _ => unreachable!(),
        }
    };*/
    let mut parse_args = |arg: Pair<Rule>| {
        let mut argument = arg.into_inner();
        let first = argument.next().unwrap();
        match first.as_rule() {
            Rule::symbol => ast::Argument {
                symbol: Some(first.as_span().as_str().to_string()),
                value: parse_expr(argument.next().unwrap(), notes),
            },
            Rule::expr => ast::Argument {
                symbol: None,
                value: parse_expr(first, notes),
            },
            _ => unreachable!(),
        }
    };
    match pair.as_rule() {
        Rule::symbol => ast::Path::Member(pair.as_span().as_str().to_string()),

        Rule::index => ast::Path::Index(parse_expr(pair.into_inner().next().unwrap(), notes)),

        Rule::arguments => ast::Path::Call(pair.into_inner().map(|x| parse_args(x)).collect()),

        _ => unreachable!(),
    }
}

pub fn parse_variable(pair: Pair<Rule>, notes: &mut ParseNotes) -> ast::Variable {
    let mut call_list = pair.into_inner();
    let first = call_list.next().unwrap();

    let value: ast::ValueLiteral;

    let operator = match first.as_rule() {
        Rule::unary_operator => {
            value = parse_value(call_list.next().unwrap(), notes);
            Some(first.as_span().as_str().to_string())
        }
        _ => {
            value = parse_value(first, notes);
            None
        }
    };
    let path: Vec<ast::Path> = call_list.map(|x| parse_path(x, notes)).collect();
    fn parse_value(pair: Pair<Rule>, notes: &mut ParseNotes) -> ast::ValueLiteral {
        match pair.as_rule() {
            Rule::id => {
                let number: u16;
                let mut scope = pair.into_inner();
                let mut unspecified = false;
                let first_value = scope.next().unwrap();
                let class_name: String;

                if first_value.as_rule() == Rule::number {
                    number = first_value.as_span().as_str().parse().unwrap();
                    class_name = scope.next().unwrap().as_span().as_str().to_string();

                    match class_name.as_ref() {
                        "g" => (*notes).closed_groups.push(number),
                        "c" => (*notes).closed_colors.push(number),
                        "b" => (*notes).closed_blocks.push(number),
                        "i" => (*notes).closed_items.push(number),
                        _ => unreachable!(),
                    }
                } else {
                    unspecified = true;
                    number = 0;
                    class_name = first_value.as_span().as_str().to_string();
                }

                ast::ValueLiteral::ID(ast::ID {
                    number,
                    unspecified,
                    class_name,
                })
            }

            Rule::macro_def => {
                let mut inner = pair.into_inner();
                ast::ValueLiteral::Macro(ast::Macro {
                    args: inner
                        .next()
                        .unwrap()
                        .into_inner()
                        .map(|arg| {
                            let mut full = arg.into_inner();
                            let name = full.next().unwrap().as_span().as_str().to_string();
                            let default = match full.next() {
                                Some(value) => Some(parse_expr(value, notes)),
                                None => None,
                            };

                            (name, default)
                        })
                        .collect(),
                    body: ast::CompoundStatement {
                        statements: parse_statements(
                            &mut inner.next().unwrap().into_inner(),
                            notes,
                        ),
                    },
                })
            }

            Rule::number => {
                ast::ValueLiteral::Number(pair.as_span().as_str().parse().expect("invalid number"))
            }

            Rule::bool => ast::ValueLiteral::Bool(pair.as_span().as_str() == "true"),

            Rule::dictionary => ast::ValueLiteral::Dictionary(parse_dict(pair, notes)),

            Rule::cmp_stmt => ast::ValueLiteral::CmpStmt(ast::CompoundStatement {
                statements: parse_statements(&mut pair.into_inner(), notes),
            }),

            Rule::obj => ast::ValueLiteral::Obj(
                pair.into_inner()
                    .map(|prop| {
                        let mut inner = prop.into_inner();
                        (
                            parse_expr(inner.next().unwrap(), notes),
                            parse_expr(inner.next().unwrap(), notes),
                        )
                    })
                    .collect(),
            ),

            Rule::value_literal => parse_value(pair.into_inner().next().unwrap(), notes),
            Rule::variable => parse_value(pair.into_inner().next().unwrap(), notes),
            Rule::expr => ast::ValueLiteral::Expression(parse_expr(pair, notes)),
            Rule::symbol => ast::ValueLiteral::Symbol(pair.as_span().as_str().to_string()),
            Rule::string => {
                ast::ValueLiteral::Str(ast::str_content(pair.as_span().as_str().to_string()))
            }
            Rule::array => {
                ast::ValueLiteral::Array(pair.into_inner().map(|x| parse_expr(x, notes)).collect())
            }
            Rule::import => ast::ValueLiteral::Import(PathBuf::from(ast::str_content(
                pair.into_inner()
                    .next()
                    .unwrap()
                    .as_span()
                    .as_str()
                    .to_string(),
            ))),
            _ => {
                println!("{:?} is not added to parse_values yet", pair.as_rule());
                ast::ValueLiteral::Number(0.0)
            }
        }
    }

    ast::Variable {
        value,
        path,
        operator,
    }
}

fn parse_dict(pair: Pair<Rule>, notes: &mut ParseNotes) -> Vec<ast::DictDef> {
    let mut out: Vec<ast::DictDef> = Vec::new();
    for def in pair.into_inner() {
        match def.as_rule() {
            Rule::dict_def => {
                let mut inner = def.into_inner();
                out.push(ast::DictDef::Def((
                    inner.next().unwrap().as_span().as_str().to_string(), //symbol
                    parse_expr(inner.next().unwrap(), notes),             //expression
                )));
            }

            Rule::dict_extract => out.push(ast::DictDef::Extract(parse_expr(
                def.into_inner().next().unwrap(),
                notes,
            ))),

            _ => unreachable!(),
        };
    }
    out
}

fn parse_expr(pair: Pair<Rule>, notes: &mut ParseNotes) -> ast::Expression {
    let mut values: Vec<ast::Variable> = Vec::new();
    let mut operators: Vec<String> = Vec::new();

    for item in pair.into_inner() {
        match item.as_rule() {
            Rule::operator => operators.push(item.as_span().as_str().to_string()),
            _ => values.push(parse_variable(item, notes)),
        }
    }

    ast::Expression { operators, values }
}

/*#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decrypt() {
        //not working on mac :(
        let file_content = fs::read_to_string(
            "/Users/August/Library/Application Support/GeometryDash/CCLocalLevels.dat",
        )
        .expect("Something went wrong reading the file");
        println!(
            "{}",
            levelstring::get_level_string(file_content.to_string())
        );
    }
}*/
*/
