use crate::method;

use super::{
    interpreter::{Globals, ValueKey},
    types::TypeBuilder,
    value::ValueType,
};

impl Globals {
    pub fn init_types(&mut self) {
        TypeBuilder::<Vec<ValueKey>>::new(ValueType::Array)
            .add_member(self, "length", |_, this| this.len())
            // .add_method(
            //     &mut self,
            //     method! {
            //         push(mut this, el: Int) => self.push(el)
            //     },
            // )
            .add_method(self, "test", |_, vals| Ok(vals[0]))
            .finish_type(self);
    }
}
