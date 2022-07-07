use std::fmt;
use std::sync::Arc;

use ahash::AHashMap;
use serde::{Deserialize, Serialize};

use super::from_value::{Error, FromValueList};
use super::interpreter::Globals;
use super::method::{Function, Method};
use super::to_value::ToValueResult;
use super::types::Instance;
use super::value::Value;

type StaticMethodType<T> = Arc<dyn Fn(Vec<Value>) -> Result<T, Error> + Send + Sync>;
type SelfMethodType<T> =
    Arc<dyn Fn(&Instance, Vec<Value>, &mut Globals) -> Result<T, Error> + Send + Sync>;

// `SelfMethod` i.e. a instance method (where `self` is the first argument)
#[derive(Serialize, Deserialize, Clone)]
pub struct SelfMethod(SelfMethodType<Value>);

impl SelfMethod {
    pub fn new<T, F, Args>(f: F) -> Self
    where
        Args: FromValueList,
        F: Method<T, Args>,
        F::Result: ToValueResult,
        T: 'static,
    {
        Self(Arc::new(
            move |instance: &Instance, args: Vec<Value>, globals: &mut Globals| {
                let instance = instance.downcast(Some(globals));

                let args = Args::from_value_list(&args);

                instance
                    .and_then(|i| args.map(|a| (i, a)))
                    .and_then(|(instance, args)| f.invoke(instance, args).to_value_result())
            },
        ))
    }

    pub fn from_static_method(method: StaticMethod) -> Self {
        Self(Arc::new(
            move |_: &Instance, args: Vec<Value>, _: &mut Globals| method.invoke(args),
        ))
    }

    pub fn invoke(
        &self,
        instance: &Instance,
        args: Vec<Value>,
        globals: &mut Globals,
    ) -> Result<Value, Error> {
        self.0(instance, args, globals)
    }
}

// need to manually implement cause the traits dont implement debug
impl fmt::Debug for SelfMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SelfMethod(...)")
    }
}

pub type SelfMethods = AHashMap<String, SelfMethod>;

// `StaticMethod` where `self` isnt the first argument
#[derive(Serialize, Deserialize, Clone)]
pub struct StaticMethod(StaticMethodType<Value>);

impl StaticMethod {
    pub fn new<F, Args>(f: F) -> Self
    where
        Args: FromValueList,
        F: Function<Args>,
        F::Result: ToValueResult,
    {
        Self(Arc::new(move |args: Vec<Value>| {
            Args::from_value_list(&args).and_then(|args| f.invoke(args).to_value_result())
        }))
    }

    pub fn invoke(&self, args: Vec<Value>) -> Result<Value, Error> {
        self.0(args)
    }
}

impl fmt::Debug for StaticMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StaticMethod(...)")
    }
}

pub type StaticMethods = AHashMap<String, StaticMethod>;

#[derive(Clone)]
pub struct AttributeGetter(
    Arc<dyn Fn(&Instance, &mut Globals) -> Result<Value, Error> + Send + Sync>,
);

impl AttributeGetter {
    pub fn new<T, F, R>(f: F) -> Self
    where
        T: 'static,
        F: Fn(&T) -> R + Send + Sync + 'static,
        R: ToValueResult,
    {
        Self(Arc::new(move |instance, globals: &mut Globals| {
            let instance = instance.downcast(Some(globals));
            instance.map(&f).and_then(|v| v.to_value_result())
        }))
    }

    pub fn invoke(&self, instance: &Instance, globals: &mut Globals) -> Result<Value, Error> {
        self.0(instance, globals)
    }
}

impl fmt::Debug for AttributeGetter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AttributeGetter(...)")
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
        Constructor(Arc::new(move |args: Vec<Value>| {
            Args::from_value_list(&args).map(|args| {
                let s = f.invoke(args);
                Instance::new(s, s.name)
            })
        }))
    }

    pub fn invoke(&self, args: Vec<Value>) -> Result<Instance, Error> {
        self.0(args)
    }
}

impl fmt::Debug for Constructor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Constructor(...)")
    }
}
