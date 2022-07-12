use std::sync::Arc;

use ahash::AHashMap;

use super::error::RuntimeError;
use super::from_value::FromValueList;
use super::interpreter::Globals;
use super::method::{Function, Method};
use super::to_value::ToValueResult;
use super::types::Instance;
use super::value::Value;

use crate::sources::CodeArea;

// type StaticMethodType<T> =
//     Arc<dyn Fn(Vec<Value>, CodeArea) -> Result<T, RuntimeError> + Send + Sync>;
// type SelfMethodType<'smt, T> = Arc<
//     dyn Fn(&'smt Instance, Vec<Value>, &mut Globals, CodeArea) -> Result<T, RuntimeError>
//         + Send
//         + Sync,
// >;

// // `SelfMethod` i.e. a instance method (where `self` is the first argument)
// #[derive(Clone)]
// pub struct SelfMethod<'sm>(SelfMethodType<'sm, Value>);

// impl<'sm> SelfMethod<'sm> {
//     pub fn new<'a, T, F, Args>(f: F) -> Self
//     where
//         Args: FromValueList,
//         F: Method<T, Args>,
//         F::Result: ToValueResult,
//         T: Send + Sync + 'sm,
//         'a: 'sm,
//     {
//         Self(Arc::new(
//             move |instance: &'sm Instance,
//                   args: Vec<Value>,
//                   globals: &mut Globals,
//                   area: CodeArea| {
//                 let instance = instance.downcast::<T>();

//                 let args = Args::from_value_list(&args, area)?;

//                 f.invoke(instance, args).try_to_value()
//                 // instance
//                 //     .and_then(|i| args.map(|a| (i, a)))
//                 //     .and_then(|(instance, args)| f.invoke(instance, args).try_to_value())
//             },
//         ))
//     }

//     pub fn from_static_method(method: StaticMethod) -> Self {
//         Self(Arc::new(
//             move |_: &Instance, args: Vec<Value>, _: &mut Globals, area: CodeArea| {
//                 method.invoke(args, area)
//             },
//         ))
//     }

//     pub fn invoke(
//         &self,
//         instance: &'sm Instance,
//         args: Vec<Value>,
//         globals: &mut Globals,
//         area: CodeArea,
//     ) -> Result<Value, RuntimeError> {
//         self.0(instance, args, globals, area)
//     }
// }
// pub type SelfMethods<'sm> = AHashMap<String, SelfMethod<'sm>>;

// // `StaticMethod` where `self` isnt the first argument
// #[derive(Clone)]
// pub struct StaticMethod(StaticMethodType<Value>);

// impl StaticMethod {
//     pub fn new<F, Args>(f: F) -> Self
//     where
//         Args: FromValueList,
//         F: Function<Args>,
//         F::Result: ToValueResult,
//     {
//         Self(Arc::new(move |args: Vec<Value>, area: CodeArea| {
//             Args::from_value_list(&args, area).and_then(|args| f.invoke(args).try_to_value())
//         }))
//     }

//     pub fn invoke(&self, args: Vec<Value>, area: CodeArea) -> Result<Value, RuntimeError> {
//         self.0(args, area)
//     }
// }
// pub type StaticMethods = AHashMap<String, StaticMethod>;

// #[derive(Clone)]
// pub struct AttributeGetter(
//     Arc<dyn Fn(&Instance, &mut Globals) -> Result<Value, RuntimeError> + Send + Sync>,
// );

// impl<'ag> AttributeGetter {
//     pub fn new<T, F, R>(f: F) -> Self
//     where
//         T: Send + Sync + 'ag,
//         F: Fn(&T) -> R + Send + Sync + 'ag,
//         R: ToValueResult,
//     {
//         Self(Arc::new(move |instance, globals: &mut Globals| {
//             let instance = instance.downcast::<T>();
//             //instance.map(&f).and_then(|v| v.try_to_value())
//             (&f)(instance).try_to_value()
//         }))
//     }

//     pub fn invoke(
//         &self,
//         instance: &Instance,
//         globals: &mut Globals,
//     ) -> Result<Value, RuntimeError> {
//         self.0(instance, globals)
//     }
// }

// pub type Attributes = AHashMap<String, AttributeGetter>;

// #[derive(Clone)]
// pub struct Constructor<'c>(StaticMethodType<Instance<'c>>);

// impl<'c> Constructor<'c> {
//     pub fn new<Args, F>(f: F) -> Self
//     where
//         Args: FromValueList,
//         F: Function<Args>,
//         F::Result: Send + Sync + 'c,
//     {
//         Constructor(Arc::new(move |args: Vec<Value>, area: CodeArea| {
//             Args::from_value_list(&args, area).map(|args| {
//                 let s = f.invoke(args);
//                 Instance::new(s, todo!("test"))
//             })
//         }))
//     }

//     pub fn invoke(&self, args: Vec<Value>, area: CodeArea) -> Result<Instance<'c>, RuntimeError> {
//         self.0(args, area)
//     }
// }

type StaticMethodType<T> =
    Arc<dyn Fn(Vec<Value>, CodeArea) -> Result<T, RuntimeError> + Send + Sync>;
type SelfMethodType<T> = Arc<
    dyn Fn(&Instance, Vec<Value>, &mut Globals, CodeArea) -> Result<T, RuntimeError> + Send + Sync,
>;

// `SelfMethod` i.e. a instance method (where `self` is the first argument)
#[derive(Clone)]
pub struct SelfMethod(SelfMethodType<Value>);

impl SelfMethod {
    pub fn new<'a, T, F, Args>(f: F) -> Self
    where
        Args: FromValueList,
        F: Method<T, Args>,
        F::Result: ToValueResult,
        T: Send + Sync + 'static,
    {
        Self(Arc::new(
            move |instance: &Instance, args: Vec<Value>, globals: &mut Globals, area: CodeArea| {
                let instance = instance.downcast::<T>();

                let args = Args::from_value_list(&args, area)?;

                f.invoke(instance, args).try_to_value()
                // instance
                //     .and_then(|i| args.map(|a| (i, a)))
                //     .and_then(|(instance, args)| f.invoke(instance, args).try_to_value())
            },
        ))
    }

    pub fn from_static_method(method: StaticMethod) -> Self {
        Self(Arc::new(
            move |_: &Instance, args: Vec<Value>, _: &mut Globals, area: CodeArea| {
                method.invoke(args, area)
            },
        ))
    }

    pub fn invoke(
        &self,
        instance: &Instance,
        args: Vec<Value>,
        globals: &mut Globals,
        area: CodeArea,
    ) -> Result<Value, RuntimeError> {
        self.0(instance, args, globals, area)
    }
}
pub type SelfMethods = AHashMap<String, SelfMethod>;

// `StaticMethod` where `self` isnt the first argument
#[derive(Clone)]
pub struct StaticMethod(StaticMethodType<Value>);

impl StaticMethod {
    pub fn new<F, Args>(f: F) -> Self
    where
        Args: FromValueList,
        F: Function<Args>,
        F::Result: ToValueResult,
    {
        Self(Arc::new(move |args: Vec<Value>, area: CodeArea| {
            Args::from_value_list(&args, area).and_then(|args| f.invoke(args).try_to_value())
        }))
    }

    pub fn invoke(&self, args: Vec<Value>, area: CodeArea) -> Result<Value, RuntimeError> {
        self.0(args, area)
    }
}
pub type StaticMethods = AHashMap<String, StaticMethod>;

#[derive(Clone)]
pub struct AttributeGetter(
    Arc<dyn Fn(&Instance, &mut Globals) -> Result<Value, RuntimeError> + Send + Sync>,
);

impl AttributeGetter {
    pub fn new<T, F, R>(f: F) -> Self
    where
        T: Send + Sync + 'static,
        F: Fn(&T) -> R + Send + Sync + 'static,
        R: ToValueResult,
    {
        Self(Arc::new(move |instance, globals: &mut Globals| {
            let instance = instance.downcast::<T>();
            //instance.map(&f).and_then(|v| v.try_to_value())
            (&f)(instance).try_to_value()
        }))
    }

    pub fn invoke(
        &self,
        instance: &Instance,
        globals: &mut Globals,
    ) -> Result<Value, RuntimeError> {
        self.0(instance, globals)
    }
}

pub type Attributes = AHashMap<String, AttributeGetter>;

#[derive(Clone)]
pub struct Constructor(StaticMethodType<Instance>);

impl Constructor {
    pub fn new<Args, F>(f: F) -> Self
    where
        Args: FromValueList,
        F: Function<Args>,
        F::Result: Send + Sync + 'static,
    {
        Constructor(Arc::new(move |args: Vec<Value>, area: CodeArea| {
            Args::from_value_list(&args, area).map(|args| {
                let s = f.invoke(args);
                Instance::new(s, todo!("test"))
            })
        }))
    }

    pub fn invoke(&self, args: Vec<Value>, area: CodeArea) -> Result<Instance, RuntimeError> {
        self.0(args, area)
    }
}
