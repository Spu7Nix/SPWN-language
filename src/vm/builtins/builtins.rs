use std::io;
use std::io::Write;
use std::ops::Range;

use delve::{EnumDisplay, EnumFromStr};
use rand::seq::SliceRandom;
use rand::Rng;

use super::builtin_utils::{Invoke, Spread, TOf, ToValue};
use crate::of;
use crate::sources::CodeArea;
use crate::vm::error::RuntimeError;
use crate::vm::interpreter::{ValueKey, Vm};
use crate::vm::value::Value;
use crate::vm::value_ops;

#[derive(Debug, EnumFromStr, EnumDisplay, PartialEq, Clone)]
#[delve(rename_variants = "snake_case")]
pub enum Builtin {
    Print,
    Println,
    Exit,
    Random,
    Version,
    Assert,
    AssertEq,
    Input,
    Add,
}

impl Builtin {
    pub fn call(
        &self,
        args: &mut Vec<ValueKey>,
        vm: &mut Vm,
        area: CodeArea,
    ) -> Result<Value, RuntimeError> {
        match self {
            Self::Print => print.invoke_fn(args, vm, area).to_value(vm),
            Self::Println => println.invoke_fn(args, vm, area).to_value(vm),
            Self::Exit => exit.invoke_fn(args, vm, area).to_value(vm),
            //Self::Random => random.invoke_fn(args, vm, area).to_value(vm),
            Self::Version => version.invoke_fn(args, vm, area).to_value(vm),
            // Self::Input => input.invoke_fn(args, vm, area).to_value(vm),
            Self::Assert => assert.invoke_fn(args, vm, area).to_value(vm),
            Self::AssertEq => assert_eq.invoke_fn(args, vm, area).to_value(vm),
            Self::Add => add.invoke_fn(args, vm, area).to_value(vm),
            _ => todo!(),
        }
    }
}

pub fn add(object: Value, vm: &mut Vm) {
    // vm.contexts.yeet_current(
    // the goof (the sill)
    println!("helloe!!!");
}

pub fn exit(_vm: &mut Vm) {
    // vm.contexts.yeet_current();
    // the goof (the sill)
}

pub fn print(values: Spread<Value>, vm: &Vm) {
    print!(
        "{}",
        values
            .iter()
            .map(|v| v.runtime_display(vm))
            .collect::<Vec<_>>()
            .join(" ")
    )
}
pub fn println(values: Spread<Value>, vm: &Vm) {
    println!(
        "{}",
        values
            .iter()
            .map(|v| v.runtime_display(vm))
            .collect::<Vec<_>>()
            .join(" ")
    )
}

pub fn random(value: of!(Range<i64>, Vec<Value>, i64, f64)) -> Value {
    if let Some(range) = value.get::<Range<i64>>() {
        return Value::Int(rand::thread_rng().gen_range(range.clone()));
    }
    if let Some(values) = value.get::<Vec<Value>>() {
        // TODO: handle empty array !!!!
        return values.choose(&mut rand::thread_rng()).unwrap().clone();
    }
    if let Some(n) = value.get::<i64>() {
        return Value::Int(rand::thread_rng().gen_range(0..*n));
    }
    if let Some(n) = value.get::<f64>() {
        return Value::Float(rand::thread_rng().gen_range(0.0..*n));
    }

    unreachable!()
}

pub fn input(prompt: Option<String>) -> String {
    let prompt = prompt.unwrap_or(String::new());

    print!("{prompt}");
    std::io::stdout().flush().unwrap();

    let mut s = String::new();
    io::stdin().read_line(&mut s).expect("Couldn't read line");

    s.trim_end_matches(|p| matches!(p, '\n' | '\r')).into()
}

pub fn version() -> String {
    env!("CARGO_PKG_VERSION").into()
}

pub fn assert(expr: bool, vm: &Vm, area: CodeArea) -> Result<(), RuntimeError> {
    if !expr {
        return Err(RuntimeError::AssertionFailed {
            area,
            call_stack: vm.get_call_stack(),
        });
    }

    Ok(())
}
pub fn assert_eq(a: Value, b: Value, vm: &Vm, area: CodeArea) -> Result<(), RuntimeError> {
    if !value_ops::equality(&a, &b, vm) {
        return Err(RuntimeError::EqAssertionFailed {
            area,
            left: a.runtime_display(vm),
            right: b.runtime_display(vm),
            call_stack: vm.get_call_stack(),
        });
    }

    Ok(())
}
