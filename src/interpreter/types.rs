use std::marker::PhantomData;
use std::sync::Arc;

use super::error::RuntimeError;
use super::from_value::FromValueList;
use super::interpreter::Globals;
use super::method::{Function, Method};
use super::to_value::{ToValue, ToValueResult};
use super::type_method::{
    AttributeGetter, Attributes, Constructor, SelfMethod, SelfMethods, StaticMethod, StaticMethods,
};
use super::value::Value;

use crate::sources::CodeArea;

#[derive(Clone)]
pub struct Type {
    pub name: String,
    constructor: Option<Constructor>,
    attributes: Attributes,
    self_methods: SelfMethods,
    static_methods: StaticMethods,
}

impl Type {
    pub fn call_static(
        &self,
        name: &str,
        args: Vec<Value>,
        area: CodeArea,
    ) -> Result<Value, RuntimeError> {
        let attr = self
            .static_methods
            .get(name)
            .ok_or_else(|| RuntimeError::UndefinedMember {
                name: name.into(),
                area: area.clone(),
            })?;

        attr.clone().invoke(args, area)
    }

    pub fn get_self_method(
        &self,
        name: &String,
        area: CodeArea,
    ) -> Result<SelfMethod, RuntimeError> {
        // if the self method doesnt exist check if it's a static method as they also can be called from `self.`
        if let Some(method) = self.self_methods.get(name).cloned() {
            return Ok(method);
        }
        if let Some(method) = self.static_methods.get(name).cloned() {
            return Ok(SelfMethod::from_static_method(method));
        }
        Err(RuntimeError::UndefinedMember {
            name: name.clone(),
            area,
        })
    }
}

pub struct TypeBuilder<T> {
    typ: Type,
    ty: PhantomData<T>,
}

impl<T> TypeBuilder<T>
where
    T: Send + Sync + 'static,
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

pub trait AnyUID<'a>: Send + Sync + 'a {
    fn uid(&self) -> u64;
}

impl<'a, T: Send + Sync + Sized + 'a> AnyUID<'a> for T {
    fn uid(&self) -> u64 {
        self as *const T as u64
    }
}

impl<'a> dyn AnyUID<'a> + Send + Sync {
    fn downcast_ref<T: AnyUID<'a>>(&self) -> Option<&T> {
        // saftey: checking that both types are
        // TODO: does this check pass?
        //! both structs might have diff addresses
        // dbg!(self.id(), UniqueId::of::<T>());

        // if self.id() == UniqueId::of::<T>() {
        Some(unsafe { &*(self as *const Self as *const T) })
        // } else {
        //     None
        // }
    }
}

#[derive(Clone)]
pub struct Instance {
    inner: Arc<dyn AnyUID<'static> + Send + Sync>,
    name: String,
}

impl Instance {
    pub fn of(typ: &Type, fields: Vec<Value>, area: CodeArea) -> Result<Self, RuntimeError> {
        if let Some(ctor) = &typ.constructor {
            ctor.invoke(fields, area)
        } else {
            Err(RuntimeError::NoConstructor {
                typ: typ.name.clone(),
                area,
            })
        }
    }

    pub fn new<T: Send + Sync + 'static>(instance: T, name: String) -> Self {
        Self {
            inner: Arc::new(instance),
            name,
        }
    }

    pub fn inner_type<'a>(
        &self,
        globals: &'a Globals,
        area: CodeArea,
    ) -> Result<&'a Type, RuntimeError> {
        globals
            .types
            .get(&self.name)
            .ok_or_else(|| RuntimeError::UndefinedType {
                name: self.name.clone(),
                area,
            })
    }

    pub fn get_attr(
        &self,
        name: &str,
        globals: &mut Globals,
        area: CodeArea,
    ) -> Result<Value, RuntimeError> {
        let attr = self
            .inner_type(globals, area.clone())
            .and_then(|c| {
                c.attributes
                    .get(name)
                    .ok_or_else(|| RuntimeError::UndefinedMember {
                        name: name.into(),
                        area,
                    })
            })?
            .clone();

        attr.invoke(self, globals)
    }

    pub fn call_self(
        &self,
        name: String,
        args: Vec<Value>,
        globals: &mut Globals,
        area: CodeArea,
    ) -> Result<Value, RuntimeError> {
        let method = self
            .inner_type(globals, area.clone())
            .and_then(|c| c.get_self_method(&name, area.clone()))?;

        method.invoke(self, args, globals, area)
    }

    pub fn downcast<T: 'static + Send + Sync>(&self) -> &T {
        self.inner
            .as_ref()
            .downcast_ref()
            .expect("downcast error please report")
    }

    pub fn raw<T: 'static + Send + Sync>(&self) -> &T {
        self.downcast::<T>()
    }
}

// #[derive(Clone)]
// pub struct Type<'t> {
//     pub name: String,
//     constructor: Option<Constructor<'t>>,
//     attributes: Attributes,
//     self_methods: SelfMethods,
//     static_methods: StaticMethods,
// }

// impl<'t> Type<'t> {
//     pub fn call_static(
//         &self,
//         name: &str,
//         args: Vec<Value>,
//         area: CodeArea,
//     ) -> Result<Value, RuntimeError> {
//         let attr = self
//             .static_methods
//             .get(name)
//             .ok_or_else(|| RuntimeError::UndefinedMember {
//                 name: name.into(),
//                 area,
//             })?;

//         attr.clone().invoke(args, area)
//     }

//     pub fn get_self_method(
//         &self,
//         name: &String,
//         area: CodeArea,
//     ) -> Result<SelfMethod, RuntimeError> {
//         // if the self method doesnt exist check if it's a static method as they also can be called from `self.`
//         if let Some(method) = self.self_methods.get(name).cloned() {
//             return Ok(method);
//         }
//         if let Some(method) = self.static_methods.get(name).cloned() {
//             return Ok(SelfMethod::from_static_method(method));
//         }
//         Err(RuntimeError::UndefinedMember {
//             name: name.clone(),
//             area,
//         })
//     }
// }

// pub struct TypeBuilder<'tb, T> {
//     typ: Type<'tb>,
//     ty: PhantomData<T>,
// }

// impl<'tb, T> TypeBuilder<'tb, T>
// where
//     T: 'tb + Send + Sync,
// {
//     pub fn name<U: ToString>(name: U) -> Self {
//         Self {
//             typ: Type {
//                 name: name.to_string(),
//                 constructor: None,
//                 attributes: Attributes::new(),
//                 //type_id: hashed,
//                 static_methods: StaticMethods::new(),
//                 self_methods: SelfMethods::new(),
//             },
//             ty: PhantomData,
//         }
//     }

//     pub fn from(typ: Type<'tb>) -> Self {
//         Self {
//             typ,
//             ty: PhantomData,
//         }
//     }

//     pub fn add_attribute<F, R, S>(mut self, name: S, f: F) -> Self
//     where
//         F: Fn(&T) -> R + Send + Sync + 'tb,
//         R: ToValue,
//         T: 'tb,
//         S: ToString,
//     {
//         self.typ
//             .attributes
//             .insert(name.to_string(), AttributeGetter::new(f));
//         self
//     }

//     pub fn set_constructor<F, Args, R>(mut self, f: F) -> Self
//     where
//         F: Function<Args, Result = R>,
//         T: Send + Sync,
//         R: Send + Sync + 'tb,
//         Args: FromValueList,
//     {
//         self.typ.constructor = Some(Constructor::new(f));
//         self
//     }

//     pub fn add_static_method<F, Args, R, S>(mut self, name: S, f: F) -> Self
//     where
//         F: Function<Args, Result = R>,
//         Args: FromValueList,
//         R: ToValueResult + 'tb,
//         S: ToString,
//     {
//         self.typ
//             .static_methods
//             .insert(name.to_string(), StaticMethod::new(f));
//         self
//     }

//     pub fn add_self_method<F, Args, R, S>(mut self, name: S, f: F) -> Self
//     where
//         Args: FromValueList,
//         F: Method<T, Args, Result = R>,
//         R: ToValueResult + 'tb,
//         S: ToString,
//     {
//         self.typ
//             .self_methods
//             .insert(name.to_string(), SelfMethod::new(f));
//         self
//     }

//     pub fn finish(self) -> Type<'tb> {
//         self.typ
//     }
// }

// // struct UniqueId {
// //     i: u64,
// // }

// // impl UniqueId {
// //     pub const fn of<T: ?Sized>() -> UniqueId {
// //         // addresses can never be above 64 bitsă
// //         Self {
// //             i: Self as *const T,
// //         }
// //     }
// // }

// pub trait AnyUID<'a>: Send + Sync + 'a {
//     //fn uid(&self) -> UniqueId;
//     fn uid(&self) -> u64;
// }

// impl<'a, T: Send + Sync + Sized + 'a> AnyUID<'a> for T {
//     // fn uid(&self) -> UniqueId {
//     //     UniqueId::of::<T>()
//     // }
//     fn uid(&self) -> u64 {
//         self as *const T as u64
//     }
// }

// impl<'a> dyn AnyUID<'a> + Send + Sync {
//     fn downcast_ref<T: AnyUID<'a>>(&self) -> Option<&T> {
//         // saftey: checking that both types are
//         // TODO: does this check pass?
//         //! both structs might have diff addresses
//         // dbg!(self.id(), UniqueId::of::<T>());

//         // if self.id() == UniqueId::of::<T>() {
//         Some(unsafe { &*(self as *const Self as *const T) })
//         // } else {
//         //     None
//         // }
//     }
// }

// #[derive(Clone)]
// pub struct Instance<'i> {
//     inner: Arc<dyn AnyUID<'i> + Send + Sync>,
//     name: String,
// }

// impl<'i> Instance<'i> {
//     pub fn of<'a>(typ: &Type<'i>, fields: Vec<Value>, area: CodeArea) -> Result<Self, RuntimeError>
//     where
//         'a: 'i,
//     {
//         if let Some(ctor) = &typ.constructor {
//             ctor.invoke(fields, area)
//         } else {
//             Err(RuntimeError::NoConstructor {
//                 typ: typ.name,
//                 area,
//             })
//         }
//     }

//     pub fn new<T: Send + Sync + 'i>(instance: T, name: String) -> Self {
//         Self {
//             inner: Arc::new(instance),
//             name,
//         }
//     }

//     pub fn inner_type(
//         &self,
//         globals: &'i Globals<'i>,
//         area: CodeArea,
//     ) -> Result<&'i Type, RuntimeError> {
//         globals
//             .types
//             .get(&self.name)
//             .ok_or_else(|| RuntimeError::UndefinedType {
//                 name: self.name,
//                 area,
//             })
//     }

//     pub fn get_attr(
//         &'i self,
//         name: &str,
//         globals: &'i mut Globals<'i>,
//         area: CodeArea,
//     ) -> Result<Value, RuntimeError> {
//         let attr = self
//             .inner_type(globals, area)
//             .and_then(|c| {
//                 c.attributes
//                     .get(name)
//                     .ok_or_else(|| RuntimeError::UndefinedMember {
//                         name: name.into(),
//                         area,
//                     })
//             })?
//             .clone();

//         attr.invoke(self, globals)
//     }

//     pub fn call_self(
//         &'i self,
//         name: String,
//         args: Vec<Value>,
//         globals: &'i mut Globals<'i>,
//         area: CodeArea,
//     ) -> Result<Value, RuntimeError> {
//         let method = self
//             .inner_type(globals, area)
//             .and_then(|c| c.get_self_method(&name, area))?;

//         method.invoke(self, args, globals, area)
//     }

//     pub fn downcast<T: Send + Sync>(self) -> T {
//         self.inner
//             .as_ref()
//             .downcast_ref()
//             .expect("downcast error please report")
//             .clone()
//     }

//     pub fn raw<'a, T: Send + Sync + 'a>(&'i self) -> T {
//         self.downcast::<T>()
//     }
// }
