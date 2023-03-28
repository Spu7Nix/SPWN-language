use crate::sources::CodeArea;
use crate::vm::builtins::builtin_utils::impl_type;
use crate::vm::value::{Value, StoredValue};

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

        // what about visibility?
        fn size(Dict(dict) as self) -> Int {
            Value::Int(dict.len() as i64)
        }

        // TODO: visibility
        fn keys(Dict(dict) as self) -> Array {
            Value::Array(dict.keys().map(|v|
                vm.memory.insert(StoredValue {
                    value: Value::String(vm.resolve(v).chars().collect()),
                    area: CodeArea { // TODO: hire a spwn dev to fix this, since I am not a spwn dev.
                        src: crate::sources::SpwnSource::Core(Default::default()),
                        span: crate::sources::CodeSpan { start: 0, end: 0 }
                    },
                })
            ).collect())
        }
        fn values(Dict(dict) as self) -> Array {
            Value::Array(dict.values().map(|v| {
                vm.deep_clone_key_insert(v.0)
            }).collect())
        }
        fn items(Dict(dict) as self) -> Array {
            let mut items = vec![];

            for (key, value) in &dict {
                items.push(StoredValue {
                    value: Value::Array(vec![
                        vm.memory.insert(StoredValue {
                            value: Value::String(vm.resolve(key).chars().collect()),
                            area: CodeArea {
                                src: crate::sources::SpwnSource::Core(Default::default()),
                                span: crate::sources::CodeSpan { start: 0, end: 0 }
                            },
                        }),
                        vm.deep_clone_key_insert(value.0),
                    ]),
                    area: CodeArea {
                        src: crate::sources::SpwnSource::Core(Default::default()),
                        span: crate::sources::CodeSpan { start: 0, end: 0 }
                    },
                });
            }

            Value::Array(items.iter().map(|v| vm.memory.insert(v.clone())).collect())
        }
    }
}
