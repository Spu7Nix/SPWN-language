use crate::interpreting::builtins::builtin_utils::impl_type;
use crate::interpreting::value::Value;
use crate::interpreting::vm::Visibility;

impl_type! {
    impl Dict {
        Constants:

        Functions(vm, call_area):
        // // todo: not store spur??? return self ???????
        // fn insert(slf: &Dict, String(a) as key, value: ValueKey) {
        //     // let k = vm.intern(&a.iter().collect::<String>());
        //     // let mut dict = slf.get_mut_ref(vm);

        //     // dict.entry(k).or_insert()

        //     // dict.insert(k, (value, Visibility::Public));

        //     // Value::Dict(dict.clone())
        // }
    }
}
