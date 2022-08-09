use ahash::AHashMap;
use ahash::AHashSet;
use slotmap::new_key_type;
use slotmap::SlotMap;

use super::builtin_types;
use super::context::FullContext;
use super::context::SkipMode::*;
use super::error::RuntimeError;
use super::instructions;
use super::types::BuiltinFunctions;
use super::types::CustomType;
use super::value::StoredValue;
use super::value::ValueType;

use crate::compilation::code::*;
use crate::leveldata::gd_types::ArbitraryId;
use crate::leveldata::object_data::GdObj;
use crate::vm::context::ReturnType;
use crate::vm::instructions::InstrData;

use paste::paste;

new_key_type! {
    pub struct ValueKey;
    pub struct TypeKey;
    pub struct BuiltinKey;
}

pub struct Globals {
    pub memory: SlotMap<ValueKey, StoredValue>,

    pub undefined_captured: AHashSet<VarID>,
    pub arbitrary_ids: [ArbitraryId; 4],

    pub objects: Vec<GdObj>,
    pub triggers: Vec<GdObj>,
    pub types: SlotMap<TypeKey, CustomType>,
    //pub type_keys: AHashMap<String, TypeKey>,
    pub type_members: AHashMap<ValueType, AHashMap<String, ValueKey>>,
    pub builtins: SlotMap<BuiltinKey, BuiltinFunctions>,
}

impl Globals {
    pub fn new(types: SlotMap<TypeKey, CustomType>) -> Self {
        let mut g = Self {
            memory: SlotMap::default(),

            undefined_captured: AHashSet::new(),
            arbitrary_ids: [0; 4],

            objects: Vec::new(),
            triggers: Vec::new(),
            types,

            builtins: SlotMap::default(),

            type_members: AHashMap::default(),
        };
        g.init_types();
        g
    }
    pub fn key_deep_clone(&mut self, k: ValueKey) -> ValueKey {
        let val = self.memory[k].clone();
        let val = val.deep_clone(self);
        self.memory.insert(val)
    }
}

pub fn run_func(
    globals: &mut Globals,
    code: &Code,
    fn_index: usize,
    contexts: &mut FullContext,
) -> Result<(), RuntimeError> {
    let instructions = &code.funcs[fn_index].instructions;

    // set all context positions to 0
    for context in contexts.iter(IncludeReturns) {
        context.inner().pos = 0;
        assert!(context.inner().returned.is_none());
    }
    // run a function for each instruction
    macro_rules! instr_funcs {
        (
            ($contexts:ident, $instr:ident, $data:ident, $globals:ident)
            $($name:ident $(($arg:ident))?)+
        ) => {
            paste! {
                match $instr {
                    $(

                        Instruction::$name$(($arg))? => instructions::[<run_ $name:snake>]($globals, &$data, $contexts $(, *$arg)?)?,

                    )+
                }
            }
        };
    }

    'instruction_loop: loop {
        if instructions.is_empty() {
            break;
        }

        let mut finished = true;
        for context in contexts.iter(SkipReturns) {
            finished = false;

            let pos = context.inner().pos;
            let instr = &instructions[pos as usize].0;
            let data = InstrData {
                code,
                span: instructions[pos as usize].1,
            };

            instr_funcs! (
                (context, instr, data, globals)
                LoadConst(a)
                Plus
                Minus
                Mult
                Div
                Modulo
                Pow
                Negate
                Not
                Eq
                Neq
                Lt
                Lte
                Gt
                Gte
                LoadVar(a)
                SetVar(a)
                CreateVar(a)
                BuildArray(a)
                BuildDict(a)
                Jump(a)
                JumpIfFalse(a)
                PopTop
                PushEmpty
                WrapMaybe
                PushNone
                TriggerFuncCall
                PushTriggerFn
                Print
                ToIter
                IterNext(a)
                Impl(a)
                PushAnyPattern
                BuildMacro(a)
                Call(a)
                Index
                Member(a)
                TypeOf
                Return
                YeetContext
                EnterArrowStatement(a)
                EnterTriggerFunction(a)

                BuildObject(a)
                BuildTrigger(a)
                AddObject

                BuildInstance(a)
            );

            for context in context.iter(SkipReturns) {
                context.inner().pos += 1;
                if context.inner().pos >= instructions.len() as isize {
                    context.inner().returned = Some(ReturnType::Implicit);
                }
            }
        }

        if finished {
            break 'instruction_loop;
        }
    }
    if contexts
        .iter(IncludeReturns)
        .any(|c| matches!(c.inner().returned, Some(ReturnType::Explicit(_))))
    {
        contexts.yeet_implicit();
    }
    contexts.clean_yeeted();
    Ok(())
}
