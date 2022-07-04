#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(isize),
    Float(f64),
    String(String),
    Bool(bool),
    Empty,
    Array(Vec<Value>),
}
