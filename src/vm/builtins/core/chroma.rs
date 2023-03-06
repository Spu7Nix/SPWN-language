use crate::vm::builtins::builtin_utils::impl_type;
use crate::vm::error::RuntimeError;
use crate::vm::value::{StoredValue, Value};
use crate::vm::value_ops;

impl_type! {

    impl Chroma {
        Constants:
        const GAGA = Int(56);

        Functions(vm, call_area):


        fn from_rgb8(
            Int(r) as red if (>=0 & <=255),
            Int(g) as green if (>=0 & <=255),
            Int(b) as blue if (>=0 & <=255),
            Int(a) as alpha if (>=0 & <=255) = {255},
        ) -> Chroma {
            Value::Chroma { r: r as u8, g: g as u8, b: b as u8, a: a as u8 }
        }

        fn from_rgb(
            Float(r) as red if (>=0.0 & <=1.0),
            Float(g) as green if (>=0.0 & <=1.0),
            Float(b) as blue if (>=0.0 & <=1.0),
            Float(a) as alpha if (>=0.0 & <=1.0) = {1.0},
        ) -> Chroma {
            Value::Chroma { r: (r * 255.0) as u8, g: (g * 255.0) as u8, b: (b * 255.0) as u8, a: (a * 255.0) as u8 }
        }

        fn from_hsv(
            Float(h) as hue if (>=0.0 & <=1.0),
            Float(s) as saturation if (>=0.0 & <=1.0),
            Float(v) as value if (>=0.0 & <=1.0),
            Float(a) as alpha if (>=0.0 & <=1.0) = {1.0},
        ) -> Chroma {
            let (r, g, b) = crate::util::hsv_to_rgb(h, s, v);
            Value::Chroma { r, g, b, a: (a * 255.0) as u8 }
        }


        fn from_hsv2(
            Float(h) as hue if (>=0.0 & <=360.0),
            Float(s) as saturation if (>=0.0 & <=100.0),
            Float(v) as value if (>=0.0 & <=100.0),
            Float(a) as alpha if (>=0.0 & <=100.0) = {100.0},
        ) -> Chroma {
            let (r, g, b) = crate::util::hsv_to_rgb(h / 360.0, s / 100.0, v / 100.0);
            Value::Chroma { r, g, b, a: (a / 100.0 * 255.0) as u8 }
        }

        fn from_hex(
            color: String | Int,
        ) -> Chroma {

            macro_rules! error {
                ($msg:expr) => {
                    RuntimeError::InvalidHexString {
                        area: call_area,
                        msg: $msg,
                        call_stack: vm.get_call_stack()
                    }
                };
            }

            let (r, g, b, a) = match color {
                ColorValue::String(s) => crate::util::hex_to_rgb(
                    u64::from_str_radix(
                        &match s.len() {
                            3 => format!("{}{}{}{}{}{}", s[0], s[0], s[1], s[1], s[2], s[2]),
                            4 => format!("{}{}{}{}{}{}{}{}", s[0], s[0], s[1], s[1], s[2], s[2], s[3], s[3]),
                            6 => (*s).iter().collect::<String>(),
                            8 => (*s).iter().collect::<String>(),
                            _ => return Err(error!("Hex strings must be of length 3, 4, 6, or 8".into()))
                        }, 16)
                            .map_err(|_| error!("Invalid hex string".into())
                    )?
                ).unwrap(),
                ColorValue::Int(n) => crate::util::hex_to_rgb(*n as u64).ok_or(error!("Invalid hex integer".into()))?,
            };

            Value::Chroma { r, g, b, a }
        }

        fn r(
            slf: &Chroma,
            set: Float | Empty if as "(@float & >=0.0 & <=1.0) | @empty" = {()},
        ) -> Float {
            match set {
                SetValue::Float(n) => {
                    *slf.get_mut_ref(vm).r = (*n * 255.0) as u8;
                    Value::Float(*n)
                },
                SetValue::Empty(_) => Value::Float(*slf.get_ref(vm).r as f64 / 255.0),
            }
        }
        fn g(
            slf: &Chroma,
            set: Float | Empty if as "(@float & >=0.0 & <=1.0) | @empty" = {()},
        ) -> Float {
            match set {
                SetValue::Float(n) => {
                    *slf.get_mut_ref(vm).g = (*n * 255.0) as u8;
                    Value::Float(*n)
                },
                SetValue::Empty(_) => Value::Float(*slf.get_ref(vm).g as f64 / 255.0),
            }
        }
        fn b(
            slf: &Chroma,
            set: Float | Empty if as "(@float & >=0.0 & <=1.0) | @empty" = {()},
        ) -> Float {
            match set {
                SetValue::Float(n) => {
                    *slf.get_mut_ref(vm).b = (*n * 255.0) as u8;
                    Value::Float(*n)
                },
                SetValue::Empty(_) => Value::Float(*slf.get_ref(vm).b as f64 / 255.0),
            }
        }
        fn a(
            slf: &Chroma,
            set: Float | Empty if as "(@float & >=0.0 & <=1.0) | @empty" = {()},
        ) -> Float {
            match set {
                SetValue::Float(n) => {
                    *slf.get_mut_ref(vm).a = (*n * 255.0) as u8;
                    Value::Float(*n)
                },
                SetValue::Empty(_) => Value::Float(*slf.get_ref(vm).a as f64 / 255.0),
            }
        }

        fn r8(
            slf: &Chroma,
            set: Int | Empty if as "(@float & >=0 & <=255) | @empty" = {()},
        ) -> Int {
            match set {
                SetValue::Int(n) => {
                    *slf.get_mut_ref(vm).r = *n as u8;
                    Value::Int(*n)
                },
                SetValue::Empty(_) => Value::Int(*slf.get_ref(vm).r as i64),
            }
        }
        fn g8(
            slf: &Chroma,
            set: Int | Empty if as "(@float & >=0 & <=255) | @empty" = {()},
        ) -> Int {
            match set {
                SetValue::Int(n) => {
                    *slf.get_mut_ref(vm).g = *n as u8;
                    Value::Int(*n)
                },
                SetValue::Empty(_) => Value::Int(*slf.get_ref(vm).g as i64),
            }
        }
        fn b8(
            slf: &Chroma,
            set: Int | Empty if as "(@float & >=0 & <=255) | @empty" = {()},
        ) -> Int {
            match set {
                SetValue::Int(n) => {
                    *slf.get_mut_ref(vm).b = *n as u8;
                    Value::Int(*n)
                },
                SetValue::Empty(_) => Value::Int(*slf.get_ref(vm).b as i64),
            }
        }
        fn a8(
            slf: &Chroma,
            set: Int | Empty if as "(@float & >=0 & <=255) | @empty" = {()},
        ) -> Int {
            match set {
                SetValue::Int(n) => {
                    *slf.get_mut_ref(vm).a = *n as u8;
                    Value::Int(*n)
                },
                SetValue::Empty(_) => Value::Int(*slf.get_ref(vm).a as i64),
            }
        }


        fn h(
            slf: &Chroma,
            set: Float | Empty if as "(@float & >=0.0 & <=1.0) | @empty" = {()},
        ) -> Float {
            let (h, s, v) = {
                let c = slf.get_ref(vm);
                crate::util::rgb_to_hsv(*c.r as f64 / 255.0, *c.g as f64 / 255.0, *c.b as f64 / 255.0)
            };

            match set {
                SetValue::Float(n) => {
                    let mutref = slf.get_mut_ref(vm);
                    (*mutref.r, *mutref.g, *mutref.b) = crate::util::hsv_to_rgb(*n, s, v);
                    Value::Float(*n)
                },
                SetValue::Empty(_) => Value::Float(h),
            }
        }
        fn s(
            slf: &Chroma,
            set: Float | Empty if as "(@float & >=0.0 & <=1.0) | @empty" = {()},
        ) -> Float {
            let (h, s, v) = {
                let c = slf.get_ref(vm);
                crate::util::rgb_to_hsv(*c.r as f64 / 255.0, *c.g as f64 / 255.0, *c.b as f64 / 255.0)
            };

            match set {
                SetValue::Float(n) => {
                    let mutref = slf.get_mut_ref(vm);
                    (*mutref.r, *mutref.g, *mutref.b) = crate::util::hsv_to_rgb(h, *n, v);
                    Value::Float(*n)
                },
                SetValue::Empty(_) => Value::Float(s),
            }
        }
        fn v(
            slf: &Chroma,
            set: Float | Empty if as "(@float & >=0.0 & <=1.0) | @empty" = {()},
        ) -> Float {
            let (h, s, v) = {
                let c = slf.get_ref(vm);
                crate::util::rgb_to_hsv(*c.r as f64 / 255.0, *c.g as f64 / 255.0, *c.b as f64 / 255.0)
            };

            match set {
                SetValue::Float(n) => {
                    let mutref = slf.get_mut_ref(vm);
                    (*mutref.r, *mutref.g, *mutref.b) = crate::util::hsv_to_rgb(h, s, *n);
                    Value::Float(*n)
                },
                SetValue::Empty(_) => Value::Float(v),
            }
        }

        fn h2(
            slf: &Chroma,
            set: Float | Empty if as "(@float & >=0.0 & <=360.0) | @empty" = {()},
        ) -> Float {
            let (h, s, v) = {
                let c = slf.get_ref(vm);
                crate::util::rgb_to_hsv(*c.r as f64 / 255.0, *c.g as f64 / 255.0, *c.b as f64 / 255.0)
            };

            match set {
                SetValue::Float(n) => {
                    let mutref = slf.get_mut_ref(vm);
                    (*mutref.r, *mutref.g, *mutref.b) = crate::util::hsv_to_rgb(*n / 360.0, s, v);
                    Value::Float(*n)
                },
                SetValue::Empty(_) => Value::Float(h * 360.0),
            }
        }
        fn s2(
            slf: &Chroma,
            set: Float | Empty if as "(@float & >=0.0 & <=100.0) | @empty" = {()},
        ) -> Float {
            let (h, s, v) = {
                let c = slf.get_ref(vm);
                crate::util::rgb_to_hsv(*c.r as f64 / 255.0, *c.g as f64 / 255.0, *c.b as f64 / 255.0)
            };

            match set {
                SetValue::Float(n) => {
                    let mutref = slf.get_mut_ref(vm);
                    (*mutref.r, *mutref.g, *mutref.b) = crate::util::hsv_to_rgb(h, *n / 100.0, v);
                    Value::Float(*n)
                },
                SetValue::Empty(_) => Value::Float(s * 100.0),
            }
        }
        fn v2(
            slf: &Chroma,
            set: Float | Empty if as "(@float & >=0.0 & <=100.0) | @empty" = {()},
        ) -> Float {
            let (h, s, v) = {
                let c = slf.get_ref(vm);
                crate::util::rgb_to_hsv(*c.r as f64 / 255.0, *c.g as f64 / 255.0, *c.b as f64 / 255.0)
            };

            match set {
                SetValue::Float(n) => {
                    let mutref = slf.get_mut_ref(vm);
                    (*mutref.r, *mutref.g, *mutref.b) = crate::util::hsv_to_rgb(h, s, *n / 100.0);
                    Value::Float(*n)
                },
                SetValue::Empty(_) => Value::Float(v * 100.0),
            }
        }

    }
}
