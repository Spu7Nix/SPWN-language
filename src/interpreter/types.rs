use std::any::{type_name, Any};
use std::marker::PhantomData;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::from_value::{Error, FromValueList};
use super::interpreter::Globals;
use super::method::{Function, Method};
use super::to_value::{ToValue, ToValueResult};
use super::type_method::{
    AttributeGetter, Attributes, Constructor, SelfMethod, SelfMethods, StaticMethod, StaticMethods,
};
use super::value::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Type {
    pub name: String,
    constructor: Option<Constructor>,
    attributes: Attributes,
    self_methods: SelfMethods,
    static_methods: StaticMethods,
}

impl Type {
    pub fn call_static(&self, name: &str, args: Vec<Value>) -> Result<Value, Error> {
        let attr = self.static_methods.get(name).ok_or_else(|| todo!())?;

        attr.clone().invoke(args)
    }

    pub fn get_self_method(&self, name: String) -> Result<SelfMethod, Error> {
        // if the self method doesnt exist check if it's a static method as they also can be called from `self.`
        if let Some(method) = self.self_methods.get(&name).cloned() {
            return Ok(method);
        }
        if let Some(method) = self.static_methods.get(&name).cloned() {
            return Ok(SelfMethod::from_static_method(method));
        }
        //Err(format!("Self method '{}' is undefined!", name))

        todo!("idk errors will happen later")
    }
}

pub struct TypeBuilder<T> {
    typ: Type,
    ty: PhantomData<T>,
}

impl<T> TypeBuilder<T>
where
    T: 'static,
{
    pub fn name<U: ToString>(name: U) -> Self {
        Self {
            typ: Type {
                name: name.to_string(),
                constructor: None,
                attributes: Attributes::new(),
                //type_id: hashed,
                static_methods: StaticMethods::new(),
                self_methods: SelfMethods::new(),
            },
            ty: PhantomData,
        }
    }

    pub fn from(typ: Type) -> Self {
        Self {
            typ,
            ty: PhantomData,
        }
    }

    pub fn add_attribute<F, R, S>(mut self, name: S, f: F) -> Self
    where
        F: Fn(&T) -> R + Send + Sync + 'static,
        R: ToValue,
        T: 'static,
        S: ToString,
    {
        self.typ
            .attributes
            .insert(name.to_string(), AttributeGetter::new(f));
        self
    }

    pub fn set_constructor<F, Args, R>(mut self, f: F) -> Self
    where
        F: Function<Args, Result = R>,
        T: Send + Sync,
        R: Send + Sync + 'static,
        Args: FromValueList,
    {
        self.typ.constructor = Some(Constructor::new(f));
        self
    }

    pub fn add_static_method<F, Args, R, S>(mut self, name: S, f: F) -> Self
    where
        F: Function<Args, Result = R>,
        Args: FromValueList,
        R: ToValueResult + 'static,
        S: ToString,
    {
        self.typ
            .static_methods
            .insert(name.to_string(), StaticMethod::new(f));
        self
    }

    pub fn add_self_method<F, Args, R, S>(mut self, name: S, f: F) -> Self
    where
        Args: FromValueList,
        F: Method<T, Args, Result = R>,
        R: ToValueResult + 'static,
        S: ToString,
    {
        self.typ
            .self_methods
            .insert(name.to_string(), SelfMethod::new(f));
        self
    }

    pub fn finish(self) -> Type {
        self.typ
    }
}
////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct Instance {
    inner: Arc<dyn Any + Send + Sync>,
    name: String,
    debug_type_name: &'static str,
}

impl Instance {
    pub fn of(typ: &Type, fields: Vec<Value>) -> Result<Self, Error> {
        if let Some(ctor) = &typ.constructor {
            ctor.invoke(fields)
        } else {
            todo!()
        }
    }

    pub fn new<T: Send + Sync + 'static>(instance: T, name: String) -> Self {
        Self {
            inner: Arc::new(instance),
            debug_type_name: type_name::<T>(),
            name,
        }
    }

    pub fn instance_of<T>(&self, typ: &Type) -> bool {
        self.name == typ.name
    }

    pub fn inner_type<'a>(&self, globals: &'a Globals) -> Result<&'a Type, Error> {
        globals.types.get(&self.type_id).ok_or_else(|| todo!())?
        //format!("Type '{:?}' is undefined!", self.debug_type_name)
    }

    pub fn name<'a>(&self, globals: &'a Globals) -> &'a str {
        self.inner_type(globals)
            .map(|ty| ty.name.as_ref())
            .unwrap_or_else(|_| self.debug_type_name)
    }

    pub fn get_attr(&self, name: &str, globals: &mut Globals) -> Result<Value, Error> {
        let attr = self
            .inner_type(globals)
            .and_then(|c| {
                c.attributes.get(name).ok_or_else(|| todo!())
                //format!("Attribute '{}' is undefined!", name)
            })?
            .clone();
        attr.invoke(self, globals)
    }

    pub fn call_self(
        &self,
        name: String,
        args: Vec<Value>,
        globals: &mut Globals,
    ) -> Result<Value, Error> {
        let method = self
            .inner_type(globals)
            .and_then(|c| c.get_self_method(name.clone()))?;
        method.invoke(self, args, globals)
    }

    pub fn downcast<T: 'static>(&self, globals: Option<&mut Globals>) -> Result<&T, Error> {
        let name = globals.as_ref().map(|g| self.name(g).to_owned()).expect(
            "tried to get inner type name of instance from a type that is not stored in globals!",
        );

        let expected_name = globals
            .as_ref()
            .and_then(|g| g.types.get(name).map(|ty| ty.name.clone()))
            .unwrap_or_else(|| self.debug_type_name.to_owned());

        self.inner.as_ref().downcast_ref().ok_or_else(|| todo!())
        //format!("Expected type '{}', got '{}'!", expected_name, name)
    }

    pub fn raw<T: Send + Sync + 'static>(&self) -> Result<&T, Error> {
        self.downcast::<T>(None)
    }
}
