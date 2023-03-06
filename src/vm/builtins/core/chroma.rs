use crate::vm::builtins::builtin_utils::impl_type;
use crate::vm::error::RuntimeError;
use crate::vm::value::{StoredValue, Value};
use crate::vm::value_ops;

impl_type! {

    impl Chroma {
        Constants:

        Functions(vm, call_area):


        fn rgb8(
            Int(r) as red if (>=0 & <=255),
            Int(g) as green if (>=0 & <=255),
            Int(b) as blue if (>=0 & <=255),
            Int(a) as alpha if (>=0 & <=255) = {255},
        ) -> Chroma {
            Value::Chroma { r: r as u8, g: g as u8, b: b as u8, a: a as u8 }
        }

        fn rgb(
            Float(r) as red if (>=0.0 & <=1.0),
            Float(g) as green if (>=0.0 & <=1.0),
            Float(b) as blue if (>=0.0 & <=1.0),
            Float(a) as alpha if (>=0.0 & <=1.0) = {1.0},
        ) -> Chroma {
            Value::Chroma { r: (r * 255.0) as u8, g: (g * 255.0) as u8, b: (b * 255.0) as u8, a: (a * 255.0) as u8 }
        }

        fn hsv(
            Float(h) as hue if (>=0.0 & <=1.0),
            Float(s) as saturation if (>=0.0 & <=1.0),
            Float(v) as value if (>=0.0 & <=1.0),
            Float(a) as alpha if (>=0.0 & <=1.0) = {1.0},
        ) -> Chroma {
            let (r, g, b) = crate::util::hsv_to_rgb(h, s, v);
            Value::Chroma { r, g, b, a: (a * 255.0) as u8 }
        }


        fn hsv2(
            Float(h) as hue if (>=0.0 & <=360.0),
            Float(s) as saturation if (>=0.0 & <=100.0),
            Float(v) as value if (>=0.0 & <=100.0),
            Float(a) as alpha if (>=0.0 & <=100.0) = {100.0},
        ) -> Chroma {
            let (r, g, b) = crate::util::hsv_to_rgb(h / 360.0, s / 100.0, v / 100.0);
            Value::Chroma { r, g, b, a: (a / 100.0 * 255.0) as u8 }
        }


    }
}
