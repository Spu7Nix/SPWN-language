use std::cell::RefCell;
use std::rc::Rc;

pub type ValueRef = Rc<RefCell<Value>>;
