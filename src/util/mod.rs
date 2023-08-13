pub mod interner;
pub(crate) mod spinner;

use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::hash_map::Drain;
use std::fmt;
use std::iter::Map;
use std::marker::PhantomData;
use std::mem::{ManuallyDrop, MaybeUninit};
use std::ops::{Index, IndexMut};
use std::path::PathBuf;
use std::process::Output;
use std::rc::Rc;

use ahash::{AHashMap, RandomState};
use bincode::de;
use colored::{ColoredString, Colorize};
use derive_more::{Deref, DerefMut};
use itertools::Itertools;
use lasso::{Rodeo, Spur};
use lazy_static::lazy_static;
use regex::Regex;
use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize};
use slab::Slab;
use widestring::{Utf32Str, Utf32String};

use crate::interpreting::error::RuntimeError;
use crate::interpreting::value::ValueType;
use crate::interpreting::vm::{RuntimeResult, Vm};
use crate::sources::CodeArea;

pub fn hyperlink<T: ToString, U: ToString>(url: T, text: Option<U>) -> String {
    let mtext = match &text {
        Some(t) => t.to_string(),
        None => url.to_string(),
    };

    match std::env::var("NO_COLOR").ok() {
        Some(_) => {
            if text.is_some() {
                format!("[{}]({mtext})", url.to_string())
            } else {
                url.to_string()
            }
        },
        None => format!("\x1B]8;;{}\x1B\\{}\x1B]8;;\x1B\\", url.to_string(), mtext)
            .blue()
            .underline()
            .bold()
            .to_string(),
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Digest(#[serde(with = "hex_serde")] [u8; 16]);

impl From<md5::Digest> for Digest {
    fn from(value: md5::Digest) -> Self {
        Self(value.0)
    }
}

pub fn hex_to_rgb(hex: u64) -> Option<(u8, u8, u8, u8)> {
    if hex > 0xffffffff {
        None
    } else if hex > 0xffffff {
        Some((
            (hex >> 24) as u8,
            ((hex % 0x1000000) >> 16) as u8,
            ((hex % 0x10000) >> 8) as u8,
            (hex % 0x100) as u8,
        ))
    } else {
        Some((
            (hex >> 16) as u8,
            ((hex % 0x10000) >> 8) as u8,
            (hex % 0x100) as u8,
            255,
        ))
    }
}

/// all values in range `0-1`
pub fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (u8, u8, u8) {
    let h = h * 6.0;

    let c = v * s;
    let x = c * (1.0 - (h.rem_euclid(2.0) - 1.0).abs());

    let (r, g, b) = if (0.0..1.0).contains(&h) {
        (c, x, 0.0)
    } else if (1.0..2.0).contains(&h) {
        (x, c, 0.0)
    } else if (2.0..3.0).contains(&h) {
        (0.0, c, x)
    } else if (3.0..4.0).contains(&h) {
        (0.0, x, c)
    } else if (4.0..5.0).contains(&h) {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    let m = v - c;
    let (r, g, b) = (r + m, g + m, b + m);

    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

pub fn rgb_to_hsv(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let x_max = r.max(g).max(b);
    let v = x_max;
    let x_min = r.min(g).min(b);

    let c = x_max - x_min;

    let h: f64 = if v == r {
        60.0 * ((g - b) / c).rem_euclid(6.0)
    } else if v == g {
        60.0 * ((b - r) / c + 2.0)
    } else if v == b {
        60.0 * ((r - g) / c + 4.0)
    } else {
        0.0
    };

    let s = if v == 0.0 { 0.0 } else { c / v };

    (h / 360.0, s, v)
}

pub trait HexColorize {
    fn color_hex(self, c: u32) -> ColoredString;
    fn on_color_hex(self, c: u32) -> ColoredString;
}

impl<T: Colorize> HexColorize for T {
    fn color_hex(self, c: u32) -> ColoredString {
        let (r, g, b, _) = hex_to_rgb(c as u64).unwrap();
        self.truecolor(r, g, b)
    }

    fn on_color_hex(self, c: u32) -> ColoredString {
        let (r, g, b, _) = hex_to_rgb(c as u64).unwrap();
        self.on_truecolor(r, g, b)
    }
}

pub struct BasicError<T: fmt::Display>(pub(crate) T);
impl<T: fmt::Display + fmt::Debug> std::error::Error for BasicError<T> {}

impl<T: fmt::Display> fmt::Debug for BasicError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T: fmt::Display> fmt::Display for BasicError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct UniqueRegister<T: std::hash::Hash + Eq> {
    map: AHashMap<T, usize>,
}

impl<T: std::hash::Hash + Eq> UniqueRegister<T> {
    pub fn new() -> Self {
        Self {
            map: AHashMap::new(),
        }
    }

    pub fn insert(&mut self, value: T) -> usize {
        let len = self.map.len();
        *self.map.entry(value).or_insert(len)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn drain(&mut self) -> impl Iterator<Item = (usize, T)> + '_ {
        self.map.drain().map(|(k, v)| (v, k))
    }
}

impl<T: std::hash::Hash + Eq> UniqueRegister<T> {
    pub fn make_vec(&mut self) -> Vec<T> {
        unsafe {
            let mut ve: Vec<MaybeUninit<T>> =
                (0..self.len()).map(|_| MaybeUninit::uninit()).collect();

            for (v, k) in self.drain() {
                ve[v].write(k);
            }

            std::mem::transmute(ve)
        }
    }
}

#[cfg(debug_assertions)]
lazy_static! {
    pub static ref BUILTIN_DIR: PathBuf = std::env::current_dir().unwrap().join("libraries");
}

#[cfg(not(debug_assertions))]
lazy_static! {
    pub static ref BUILTIN_DIR: PathBuf = home::home_dir().expect("no home dir").join(format!(
        ".spwn/versions/{}/libraries/",
        env!("CARGO_PKG_VERSION")
    ));
}

// this is equivalent to if you were to do
// something like String in the case of no mutability
// specky are you reading this if you
// a re reading this specky tag me on SPWN point server and say "Laaaaaaaa". Do it. please(it would be cool)_

pub type Str32 = Utf32Str;
pub type String32 = Utf32String;

pub type ImmutCloneStr = Rc<str>;
pub type ImmutStr = Box<str>;
pub type ImmutCloneStr32 = Rc<Str32>;
pub type ImmutStr32 = Box<Str32>;
pub type ImmutCloneVec<T> = Rc<[T]>;
pub type ImmutVec<T> = Box<[T]>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlabMap<K, V>
where
    K: From<usize>,
    usize: From<K>,
{
    slab: Slab<V>,
    _p: PhantomData<K>,
}

impl<K, V> SlabMap<K, V>
where
    K: From<usize>,
    usize: From<K>,
{
    pub fn new() -> Self {
        Self {
            slab: Slab::new(),
            _p: PhantomData,
        }
    }

    pub fn insert(&mut self, val: V) -> K {
        self.slab.insert(val).into()
    }

    pub fn get(&self, key: K) -> Option<&V> {
        self.slab.get(key.into())
    }

    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        self.slab.get_mut(key.into())
    }

    pub fn iter(&self) -> impl Iterator<Item = (K, &V)> {
        self.slab.iter().map(|(k, v)| (k.into(), v))
    }

    pub fn into_iter(self) -> impl Iterator<Item = (K, V)> {
        self.slab.into_iter().map(|(k, v)| (k.into(), v))
    }
}

impl<K, V> Index<K> for SlabMap<K, V>
where
    K: From<usize>,
    usize: From<K>,
{
    type Output = V;

    fn index(&self, index: K) -> &Self::Output {
        &self.slab[index.into()]
    }
}
impl<K, V> IndexMut<K> for SlabMap<K, V>
where
    K: From<usize>,
    usize: From<K>,
{
    fn index_mut(&mut self, index: K) -> &mut Self::Output {
        &mut self.slab[index.into()]
    }
}

#[macro_export]
macro_rules! new_id_wrapper {
    ($($name:ident : $inner:ty;)*) => {
        $(
            #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, derive_more::Deref, derive_more::DerefMut, serde::Serialize, serde::Deserialize)]
            pub struct $name(pub $inner);

            impl std::fmt::Display for $name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}({})", stringify!($name), self.0)
                }
            }

            impl From<usize> for $name {
                fn from(value: usize) -> Self {
                    Self(value as $inner)
                }
            }
            impl From<$inner> for $name {
                fn from(value: $inner) -> Self {
                    Self(value)
                }
            }

            impl From<$name> for usize {
                fn from(value: $name) -> Self {
                    value.0 as usize
                }
            }
            impl From<$name> for $inner {
                fn from(value: $name) -> Self {
                    value.0
                }
            }
        )*
    };
}

lazy_static! {
    static ref ANSI_REGEX: Regex = Regex::new(r#"(\x9B|\x1B\[)[0-?]*[ -/]*[@-~]"#).unwrap();
}

pub fn clear_ansi(s: &str) -> Cow<'_, str> {
    ANSI_REGEX.replace_all(s, "")
}

pub fn remove_quotes(s: &str) -> &str {
    &s[1..(s.len() - 1)]
}

pub fn index_wrap(
    idx: i64,
    len: usize,
    typ: ValueType,
    area: &CodeArea,
    vm: &Vm,
) -> RuntimeResult<usize> {
    let index_calc = if idx >= 0 { idx } else { len as i64 + idx };

    if index_calc < 0 || index_calc >= len as i64 {
        return Err(RuntimeError::IndexOutOfBounds {
            len,
            index: idx,
            area: area.clone(),
            typ,
            call_stack: vm.get_call_stack(),
        });
    }

    Ok(index_calc as usize)
}
