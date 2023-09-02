use ahash::AHashMap;
use lazy_static::lazy_static;
use paste::paste;

use crate::interpreting::value::ValueType;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ObjectKeyValueType {
    Int,
    Float,

    Bool,
    Group,
    Channel,
    Block,
    Item,
    GroupArray,
    String,
    Epsilon,
}

#[allow(clippy::from_over_into)]
impl Into<ValueType> for ObjectKeyValueType {
    fn into(self) -> ValueType {
        match self {
            Self::Int => ValueType::Int,
            Self::Float => ValueType::Float,
            Self::Bool => ValueType::Bool,
            Self::Group => ValueType::Group,
            Self::Channel => ValueType::Channel,
            Self::Block => ValueType::Block,
            Self::Item => ValueType::Item,
            Self::GroupArray => ValueType::Array,
            Self::String => ValueType::String,
            Self::Epsilon => ValueType::Epsilon,
        }
    }
}

macro_rules! object_keys {
    (
        $(
            $name:ident: $id:literal, $($typs:ident)|*;
        )*
    ) => {
        paste! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, delve::EnumToStr, serde::Serialize, serde::Deserialize)]
            #[delve(rename_variants = "SCREAMING_SNAKE_CASE")]
            pub enum ObjectKey {
                $(
                    [<$name:camel>],
                )*
            }

            impl ObjectKey {
                pub fn id(self) -> u8 {
                    match self {
                        $(
                            Self::[<$name:camel>] => $id,
                        )*
                    }
                }
                pub fn types(self) -> &'static [ObjectKeyValueType] {
                    match self {
                        $(
                            Self::[<$name:camel>] => &[$(ObjectKeyValueType::$typs),*],
                        )*
                    }
                }
            }

            lazy_static! {
                pub static ref OBJECT_KEYS: AHashMap<String, ObjectKey> = [
                    $((
                        stringify!($name).to_string(), ObjectKey::[<$name:camel>]
                    )),*
                ].into_iter().collect();
            }
        }
    };
}

object_keys! {
    OBJ_ID: 1, Int;
    X: 2, Float;
    Y: 3, Float;
    HORIZONTAL_FLIP: 4, Bool;
    VERTICAL_FLIP: 5, Bool;
    ROTATION: 6, Float;
    TRIGGER_RED: 7, Int;
    TRIGGER_GREEN: 8, Int;
    TRIGGER_BLUE: 9, Int;
    DURATION: 10, Float;
    TOUCH_TRIGGERED: 11, Bool;
    PORTAL_CHECKED: 13, Bool;
    PLAYER_COLOR_1: 15, Bool;
    PLAYER_COLOR_2: 16, Bool;
    BLENDING: 17, Bool;
    EDITOR_LAYER_1: 20, Int;
    COLOR: 21, Channel;
    COLOR_2: 22, Channel;
    TARGET_COLOR: 23, Channel;
    Z_LAYER: 24, Int;
    Z_ORDER: 25, Int;
    MOVE_X: 28, Int;
    MOVE_Y: 29, Int;
    EASING: 30, Int;
    TEXT: 31, String;
    SCALING: 32, Float;
    GROUP_PARENT: 34, Bool;
    OPACITY: 35, Float;
    HVS_ENABLED: 41, Bool;
    COLOR_2_HVS_ENABLED: 42, Bool;
    HVS: 43, String;
    COLOR_2_HVS: 44, String;
    FADE_IN: 45, Float;
    HOLD: 46, Float;
    FADE_OUT: 47, Float;
    PULSE_HSV: 48, Bool;
    COPIED_COLOR_HVS: 49, String;
    COPIED_COLOR_ID: 50, Channel;
    TARGET: 51, Channel | Group;
    TARGET_TYPE: 52, Bool;
    YELLOW_TELEPORTATION_PORTAL_DISTANCE: 54, Float;
    ACTIVATE_GROUP: 56, Bool;
    GROUPS: 57, Group | GroupArray;
    LOCK_TO_PLAYER_X: 58, Bool;
    LOCK_TO_PLAYER_Y: 59, Bool;
    COPY_OPACITY: 60, Bool;
    EDITOR_LAYER_2: 61, Int;
    SPAWN_TRIGGERED: 62, Bool;
    SPAWN_DURATION: 63, Float | Epsilon;
    DONT_FADE: 64, Bool;
    MAIN_ONLY: 65, Bool;
    DETAIL_ONLY: 66, Bool;
    DONT_ENTER: 67, Bool;
    ROTATE_DEGREES: 68, Float;
    TIMES_360: 69, Int;
    LOCK_OBJECT_ROTATION: 70, Bool;
    FOLLOW: 71, Group;
    CENTER: 71, Group;
    TARGET_POS: 71, Group;
    X_MOD: 72, Float;
    Y_MOD: 73, Float;
    STRENGTH: 75, Float;
    ANIMATION_ID: 76, Int;
    COUNT: 77, Int;
    SUBTRACT_COUNT: 78, Bool;
    PICKUP_MODE: 79, Int;
    ITEM: 80, Item | Block;
    BLOCK_A: 80, Block;
    HOLD_MODE: 81, Bool;
    TOGGLE_MODE: 82, Int;
    INTERVAL: 84, Float;
    EASING_RATE: 85, Float;
    EXCLUSIVE: 86, Bool;
    MULTI_TRIGGER: 87, Bool;
    COMPARISON: 88, Int;
    DUAL_MODE: 89, Bool;
    SPEED: 90, Float;
    DELAY: 91, Float;
    Y_OFFSET: 92, Int;
    ACTIVATE_ON_EXIT: 93, Bool;
    DYNAMIC_BLOCK: 94, Bool;
    BLOCK_B: 95, Block;
    GLOW_DISABLED: 96, Bool;
    ROTATION_SPEED: 97, Float;
    DISABLE_ROTATION: 98, Bool;
    COUNT_MULTI_ACTIVATE: 104, Bool;
    USE_TARGET: 100, Bool;
    TARGET_POS_AXES: 101, Int;
    EDITOR_DISABLE: 102, Bool;
    HIGH_DETAIL: 103, Bool;
    MAX_SPEED: 105, Float;
    RANDOMIZE_START: 106, Bool;
    ANIMATION_SPEED: 107, Float;
    LINKED_GROUP: 108, Int;
    ACTIVE_TRIGGER: 36, Bool;
}
