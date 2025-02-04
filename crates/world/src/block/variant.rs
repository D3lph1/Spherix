use std::ops::Deref;

/// High-performance CamelCase to snake_case converter implementation.
/// It is not general-purpose function because it assumes that input
/// string contains only ASCII characters.
fn to_snake(s: &str) -> String {
    // Using byte array instead of chars() iterator significantly (twicely)
    // improves performance of the function.
    let chars = s.as_bytes();

    // Set initial capacity to slightly increased length of input string
    // It prevents extra memory allocation.
    //                                            |<---------------->|
    let mut result = String::with_capacity(s.len() + 1);
    let ch = chars[0] as char;
    result.push(ch.to_ascii_lowercase());

    for i in 1..s.len() {
        let ch = chars[i] as char;

        if ch.is_ascii_uppercase() {
            result.push('_');
            result.push(ch.to_ascii_lowercase());
        } else {
            result.push(ch);
        }
    }

    result
}

macro_rules! variant_enum {
    (
        $(
            $variant:ident {
                $($case:ident),*
            }
        )*
    ) => {
        $(
            #[derive(PartialEq, Eq, Hash, Debug, spherix_macro::Cases)]
            pub enum $variant {
                $(
                    $case,
                )*
            }

            impl $variant {
                pub fn is_covers_all_values(list: &serde_json::Value) -> bool {
                    if let serde_json::Value::Array(values) = list {
                        let mut cases: std::collections::HashMap<_, _> = $variant::cases().into_iter().map(|k| (k, ())).collect();

                        if cases.len() != values.len() {
                            return false
                        }

                        for value in values {
                            if let serde_json::Value::String(s) = value {
                                let num: Result<$variant, _> = std::str::FromStr::from_str(s);

                                if num.is_err() {
                                    return false
                                }

                                cases.remove(&num.unwrap());
                            }
                        }

                        return cases.len() == 0
                    }

                    panic!("Not a list")
                }

                fn variant_to_snake(variant: $variant) -> &'static str {
                    match variant {
                        $(
                            $variant::$case => spherix_macro::to_snake!($case),
                        )*
                    }
                }
            }

            impl std::str::FromStr for $variant {
                type Err = ();

                fn from_str(input: &str) -> Result<$variant, Self::Err> {
                    return match to_snake(&input) {
                        $(
                            x if x == Self::variant_to_snake($variant::$case) => Ok::<_, Self::Err>($variant::$case),
                        )*
                        _ => Err(())
                    }
                }
            }
        )*
    }
}

macro_rules! variant_u8_from_str_impl {
    ($variant:ident) => {
        impl std::str::FromStr for $variant {
            type Err = core::num::ParseIntError;

            fn from_str(input: &str) -> Result<$variant, Self::Err> {
                Ok($variant::try_new(u8::from_str(input)?).expect(&format!("u8: {} {}", stringify!($variant), input)))
            }
        }
    }
}

macro_rules! variant_u8_is_covers_all_values_impl {
    ($min:literal, $max:literal) => {
        pub fn is_covers_all_values(list: &serde_json::Value) -> bool {
            if let serde_json::Value::Array(values) = list {
                let mut range: std::collections::HashMap<_, _> = ($min..$max + 1).map(|k| (k, ())).collect();

                if range.len() != values.len() {
                    return false
                }

                for value in values {
                    if let serde_json::Value::String(s) = value {
                        let num: Result<u8, _> = std::str::FromStr::from_str(s);
                        range.remove(&num.unwrap());
                    }
                }

                return range.len() == 0
            }

            panic!("Not a list")
        }
    }
}

macro_rules! variant_u8 {
    (
        $(
            $variant:ident($min:literal, $max:literal)
        ),*
    ) => {
        $(
            #[derive(PartialEq, Eq, Hash, Debug)]
            pub struct $variant(u8);

            impl $variant {
                pub fn try_new(val: u8) -> Option<Self> {
                    if val >= $min && val <= $max {
                        return Some(Self(val))
                    }

                    None
                }

                variant_u8_is_covers_all_values_impl!($min, $max);
            }

            variant_u8_from_str_impl!($variant);
        )*
    }
}

macro_rules! variant_u8_max {
    (
        $(
            $variant:ident($max:literal)
        ),*
    ) => {
        $(
            #[derive(PartialEq, Eq, Hash, Debug)]
            pub struct $variant(u8);

            impl $variant {
                pub fn try_new(val: u8) -> Option<Self> {
                    if val <= $max {
                        return Some(Self(val))
                    }

                    None
                }

                variant_u8_is_covers_all_values_impl!(0, $max);
            }

            variant_u8_from_str_impl!($variant);
        )*
    }
}

macro_rules! variant_bool {
    (
        $(
            $variant:ident
        ),*
    ) => {
        $(
            #[derive(PartialEq, Eq, Hash, Debug)]
            pub struct $variant(pub bool);

            impl $variant {
                pub fn is_covers_all_values(list: &serde_json::Value) -> bool {
                    if let serde_json::Value::Array(values) = list {
                        let mut range = std::collections::HashMap::from([
                            (false, ()),
                            (true, ())
                        ]);

                        if range.len() != values.len() {
                            return false
                        }

                        for value in values {
                            if let serde_json::Value::String(s) = value {
                                let boolean: Result<bool, _> = std::str::FromStr::from_str(s);
                                range.remove(&boolean.unwrap());
                            }
                        }

                        return range.len() == 0
                    }

                    panic!("Not a list")
                }
            }

            impl std::str::FromStr for $variant {
                type Err = core::str::ParseBoolError;

                fn from_str(input: &str) -> Result<$variant, Self::Err> {
                    Ok($variant(bool::from_str(input)?))
                }
            }
        )*
    }
}

variant_enum!(
    Face {
        Floor,
        Wall,
        Ceiling
    }

    Facing {
        North,
        South,
        West,
        East
    }

    HopperFacing {
        Down,
        North,
        South,
        West,
        East
    }

    ExtendedFacing {
        North,
        South,
        West,
        East,
        Up,
        Down
    }

    // Usually belongs to doors or to fern
    UlHalf {
        Upper,
        Lower
    }

    // Usually belongs to stairs or to trapdoor
    TbHalf {
        Top,
        Bottom
    }

    Hinge {
        Left,
        Right
    }

    Axis {
        X,
        Y,
        Z
    }

    SlabType {
        Top,
        Bottom,
        Double
    }

    PistonType {
        Normal,
        Sticky
    }

    ChestType {
        Single,
        Left,
        Right
    }

    StairsShape {
        Straight,
        InnerLeft,
        InnerRight,
        OuterLeft,
        OuterRight
    }

    RailsShape {
        NorthSouth,
        EastWest,
        AscendingEast,
        AscendingWest,
        AscendingNorth,
        AscendingSouth,
        SouthEast,
        SouthWest,
        NorthWest,
        NorthEast
    }

    PoweredRailsShape {
        NorthSouth,
        EastWest,
        AscendingEast,
        AscendingWest,
        AscendingNorth,
        AscendingSouth
    }

    East {
        None,
        Low,
        Tall
    }

    North {
        None,
        Low,
        Tall
    }

    South {
        None,
        Low,
        Tall
    }

    West {
        None,
        Low,
        Tall
    }

    RedstoneWireEast {
        Up,
        Side,
        None
    }

    RedstoneWireNorth {
        Up,
        Side,
        None
    }

    RedstoneWireSouth {
        Up,
        Side,
        None
    }

    RedstoneWireWest {
        Up,
        Side,
        None
    }

    Instrument {
        Harp,
        Basedrum,
        Snare,
        Hat,
        Bass,
        Flute,
        Bell,
        Guitar,
        Chime,
        Xylophone,
        IronXylophone,
        CowBell,
        Didgeridoo,
        Bit,
        Banjo,
        Pling,
        Zombie,
        Skeleton,
        Creeper,
        Dragon,
        WitherSkeleton,
        Piglin,
        CustomHead
    }

    Part {
        Head,
        Foot
    }

    StructureBlockMode {
        Save,
        Load,
        Corner,
        Data
    }

    SculkSensorPhase {
        Inactive,
        Active,
        Cooldown
    }

    Thickness {
        TipMerge,
        Tip,
        Frustum,
        Middle,
        Base
    }

    VerticalDirection {
        Up,
        Down
    }

    Tilt {
        None,
        Unstable,
        Partial,
        Full
    }

    Leaves {
        None,
        Small,
        Large
    }

    Orientation {
        DownEast,
        DownNorth,
        DownSouth,
        DownWest,
        UpEast,
        UpNorth,
        UpSouth,
        UpWest,
        WestUp,
        EastUp,
        NorthUp,
        SouthUp
    }

    Attachment {
        Floor,
        Ceiling,
        SingleWall,
        DoubleWall
    }

    ComparatorMode {
        Compare,
        Subtract
    }
);

variant_u8_max!(
    Rotation(15),
    Stage(1),
    Age25(25),
    Age7(7),
    Age3(3),
    Age2(2),
    Age1(1),
    Age4(4),
    Age5(5),
    Age15(15),
    Power(15),
    Note(24),
    ScaffoldingDistance(7),
    Level(15),
    Hatch(2),
    Dusted(3),
    Charges(4),
    HoneyLevel(5),
    Moisture(7),
    Bites(6)
);

variant_u8!(
    LeavesDistance(1, 7),
    Candles(1, 4),
    Eggs(1, 4),
    Layers(1, 8),
    Pickles(1, 4),
    Delay(1, 4),
    FlowerAmount(1, 4)
);

variant_bool!(
    Powered,
    Open,

    BoolNorth,
    BoolSouth,
    BoolWest,
    BoolEast,

    Waterlogged,
    InWall,
    Attached,
    Persistent,
    Up,
    Down,
    Bottom,
    Berries,
    Occupied,
    Lit,
    Disarmed,
    SignalFire,
    CanSummon,
    Shrieking,
    Locked,
    Extended,
    Hanging,
    Slot0occupied,
    Slot1occupied,
    Slot2occupied,
    Slot3occupied,
    Slot4occupied,
    Slot5occupied,
    Inverted,
    Short,
    Triggered,
    Conditional,
    HasBottle0,
    HasBottle1,
    HasBottle2,
    Enabled,
    Eye,
    HasBook,
    Snowy,
    Bloom,
    Drag
);

/// This macro does main logic for creating Variant from JSON.
/// The macro is optimized to do as less as possible: it is split into two
/// parts. The first part is used for ambiguous case (when multiple Variant
/// values have the same JSON field name). For example, both UlHalf and
/// TbHalf use field named "half". The second part, in its turn, is used
/// for deterministic scenario.
macro_rules! variant_do {
    (
        $prop_name:ident $prop_value:ident $possible_values:ident $vec:ident $variant:ident $prop:literal
    ) => {
        if $prop_name == $prop {
            if let serde_json::Value::Object(map) = $possible_values {
                if let Some(values) = map.get($prop) {
                    if <$variant>::is_covers_all_values(values) {
                        let variant: Result<$variant, _> = std::str::FromStr::from_str($prop_value);

                        $vec.push(
                            Self::$variant(
                                variant.expect(&format!("Can not unwrap property named \"{}\" with value \"{}\" for variant \"{}\" (ambiguous case)", $prop_name, $prop_value, stringify!($variant)))
                            )
                        );
                    }
                }
            }
        }
    };
    (
        $prop_name:ident $prop_value:ident $possible_values:ident $vec:ident $variant:ident
    ) => {
        let prop = spherix_macro::to_snake!($variant);

        if $prop_name == prop {
            let variant: Result<$variant, _> = std::str::FromStr::from_str($prop_value);

            $vec.push(
                Self::$variant(
                    variant.expect(&format!("Can not unwrap property named \"{}\" with value \"{}\" for variant \"{}\" (ambiguous case)", $prop_name, $prop_value, stringify!($variant)))
                )
            );
        }
    }
}

macro_rules! variant_prop {
    ($variant:ident $prop:literal) => {
        $prop
    };
    ($variant:ident) => {
        spherix_macro::to_snake!($variant)
    }
}

macro_rules! variant {
    (
        $(
            $variant:ident $($prop:literal)?
        ),*
    ) => {
        #[derive(PartialEq, Eq, Hash, Debug)]
        pub enum Variant {
            $(
                $variant($variant),
            )*
        }

        impl Variant {
            pub fn from_json_props(possible_values: &serde_json::Value, state_properties: &serde_json::Value) -> Vec<Self> {
                let mut vec = Vec::new();

                if let serde_json::Value::Object(state_properties) = state_properties {
                    for (prop_name, prop_value) in state_properties {
                        let prop_value = if let serde_json::Value::String(prop_value) = prop_value { prop_value } else { panic!("Not a string") };

                        $(
                            variant_do!(prop_name prop_value possible_values vec $variant $($prop)*);
                        )*
                    }
                }

                vec
            }

            pub fn from_nbt(nbt: &std::collections::HashMap<String, nbt::Value>, archetype: &VariantVec) -> Option<VariantVec> {
                if nbt.len() != archetype.len() {
                    return None
                }

                let mut vec = Vec::new();

                for variant in archetype.iter() {
                    let prop = nbt.get(variant.prop_name());

                    if prop.is_none() {
                        panic!()
                    }

                    let prop = if let nbt::Value::String(prop) = prop.unwrap() {prop} else {panic!()};

                    let variant = match variant {
                        $(
                            Variant::$variant(_) => {
                                let variant: $variant = std::str::FromStr::from_str(prop).unwrap();

                                Variant::$variant(variant)
                            },
                        )*
                    };

                    vec.push(variant);
                }

                Some(VariantVec::new(vec))
            }

            pub fn name(&self) -> &'static str {
                match self {
                    $(Variant::$variant(_) => stringify!($variant)),*
                }
            }

            pub fn prop_name(&self) -> &'static str {
                match self {
                    $(
                        Variant::$variant(_) => variant_prop!($variant $($prop)*),
                    )*
                }
            }
        }

        impl core::cmp::PartialOrd for Variant {
            fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
                self.name().partial_cmp(other.name())
            }
        }

        impl core::cmp::Ord for Variant {
            fn cmp(&self, other: &Self) -> core::cmp::Ordering {
                self.partial_cmp(other).unwrap()
            }
        }

        $(
            impl From<$variant> for Variant {
                fn from(value: $variant) -> Self {
                    Self::$variant(value)
                }
            }
        )*
    }
}

variant!(
    Face,
    Facing "facing",
    HopperFacing "facing",
    ExtendedFacing "facing",
    UlHalf "half",
    TbHalf "half",
    Hinge,
    Axis,
    SlabType "type",
    PistonType "type",
    ChestType "type",
    StairsShape "shape",
    RailsShape "shape",
    PoweredRailsShape "shape",
    East "east",
    North "north",
    South "south",
    West "west",
    RedstoneWireEast "east",
    RedstoneWireNorth "north",
    RedstoneWireSouth "south",
    RedstoneWireWest "west",
    Instrument,
    Part,
    StructureBlockMode "mode",
    SculkSensorPhase,
    Thickness,
    VerticalDirection,
    Tilt,
    Leaves,
    Orientation,
    Attachment,
    ComparatorMode "mode",

    Rotation,
    Stage,
    Age25 "age",
    Age7 "age",
    Age3 "age",
    Age2 "age",
    Age1 "age",
    Age4 "age",
    Age5 "age",
    Age15 "age",
    Power,
    Note,
    ScaffoldingDistance "distance",
    Level,
    Hatch,
    Dusted,
    Charges,
    HoneyLevel,
    Moisture,
    Bites,

    LeavesDistance "distance",
    Candles,
    Eggs,
    Layers,
    Pickles,
    Delay,
    FlowerAmount,

    Powered,
    Open,
    BoolNorth "north",
    BoolSouth "south",
    BoolWest "west",
    BoolEast "east",
    Waterlogged,
    InWall,
    Attached,
    Persistent,
    Up,
    Down,
    Bottom,
    Berries,
    Occupied,
    Lit,
    Disarmed,
    SignalFire,
    CanSummon,
    Shrieking,
    Locked,
    Extended,
    Hanging,
    Slot0occupied "slot_0_occupied",
    Slot1occupied "slot_1_occupied",
    Slot2occupied "slot_2_occupied",
    Slot3occupied "slot_3_occupied",
    Slot4occupied "slot_4_occupied",
    Slot5occupied "slot_5_occupied",
    Inverted,
    Short,
    Triggered,
    Conditional,
    HasBottle0 "has_bottle_0",
    HasBottle1 "has_bottle_1",
    HasBottle2 "has_bottle_2",
    Enabled,
    Eye,
    HasBook,
    Snowy,
    Bloom,
    Drag
);

/// Encapsulates vector of ordered variants
#[derive(Eq, Hash, Debug)]
pub struct VariantVec(Vec<Variant>);

impl VariantVec {
    pub fn new(mut variants: Vec<Variant>) -> Self {
        variants.sort();

        Self(variants)
    }
    
    pub fn empty() -> Self {
        Self(Vec::new())
    }
}

impl Deref for VariantVec {
    type Target = Vec<Variant>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Vec<Variant>> for VariantVec {
    fn from(value: Vec<Variant>) -> Self {
        Self::new(value)
    }
}

impl PartialEq for VariantVec {
    fn eq(&self, other: &Self) -> bool {
        for (i, variant_self) in self.iter().enumerate() {
            if variant_self != &other[i] {
                return false
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use serde_json::*;

    use crate::block::variant::to_snake as testing_to_snake;
    use crate::block::variant::*;

    #[test]
    fn to_snake() {
        assert_eq!("camel_case_string_casted_to_snake_case", testing_to_snake("CamelCaseStringCastedToSnakeCase"));
        assert_eq!("already_snake_cased_string", testing_to_snake("already_snake_cased_string"));
    }

    #[test]
    fn is_covers_all_values_enum() {
        let list = Value::Array(vec![
            "straight".into(),
            "inner_left".into(),
            "inner_right".into(),
            "outer_left".into(),
            "outer_right".into()
        ]);

        assert!(StairsShape::is_covers_all_values(&list));

        let list = Value::Array(vec![
            "straight".into(),
            "outer_left".into()
        ]);

        assert!(!StairsShape::is_covers_all_values(&list));

        let list = Value::Array(vec![
            "straight".into(),
            "inner_left".into(),
            "inner_right".into(),
            "outer_left".into(),
            "outer_right".into(),
            "another_value".into()
        ]);

        assert!(!StairsShape::is_covers_all_values(&list));
    }

    #[test]
    fn is_covers_all_values_u8_max() {
        let list = Value::Array(vec![
            "0".into(),
            "1".into(),
            "2".into(),
            "3".into(),
            "4".into(),
            "5".into(),
            "6".into(),
            "7".into()
        ]);

        assert!(Age7::is_covers_all_values(&list));

        let list = Value::Array(vec![
            "2".into(),
            "3".into(),
            "4".into()
        ]);

        assert!(!Age7::is_covers_all_values(&list));

        let list = Value::Array(vec![
            "0".into(),
            "1".into(),
            "2".into(),
            "3".into(),
            "4".into(),
            "5".into(),
            "6".into(),
            "7".into(),
            "8".into(),
            "9".into(),
        ]);

        assert!(!Age7::is_covers_all_values(&list));
    }

    #[test]
    fn is_covers_all_values_u8() {
        let list = Value::Array(vec![
            "1".into(),
            "2".into(),
            "3".into(),
            "4".into(),
            "5".into(),
            "6".into(),
            "7".into()
        ]);

        assert!(LeavesDistance::is_covers_all_values(&list));

        let list = Value::Array(vec![
            "2".into(),
            "3".into(),
            "4".into()
        ]);

        assert!(!LeavesDistance::is_covers_all_values(&list));

        let list = Value::Array(vec![
            "1".into(),
            "2".into(),
            "3".into(),
            "4".into(),
            "5".into(),
            "6".into(),
            "7".into(),
            "8".into(),
            "9".into(),
        ]);

        assert!(!LeavesDistance::is_covers_all_values(&list));
    }

    #[test]
    fn is_covers_all_values_bool() {
        let list = Value::Array(vec![
            "false".into(),
            "true".into()
        ]);

        assert!(InWall::is_covers_all_values(&list));

        let list = Value::Array(vec![
            "false".into()
        ]);

        assert!(!InWall::is_covers_all_values(&list));

        let list = Value::Array(vec![
            "false".into(),
            "true".into(),
            "intermediate".into(),
        ]);

        assert!(!InWall::is_covers_all_values(&list));
    }

    #[test]
    fn from_json_props() {
        const PROPERTIES: &str = r#"
        {
            "distance": [
                "1",
                "2",
                "3",
                "4",
                "5",
                "6",
                "7"
            ],
            "persistent": [
                "true",
                "false"
            ],
            "waterlogged": [
                "true",
                "false"
            ]
        }
        "#;

        const STATES: &str = r#"
        [
            {
                "id": 345,
                "properties": {
                    "distance": "1",
                    "persistent": "true",
                    "waterlogged": "true"
                }
            },
            {
                "id": 346,
                "properties": {
                    "distance": "1",
                    "persistent": "true",
                    "waterlogged": "false"
                }
            },
            {
                "id": 347,
                "properties": {
                    "distance": "1",
                    "persistent": "false",
                    "waterlogged": "true"
                }
            },
            {
                "id": 348,
                "properties": {
                    "distance": "1",
                    "persistent": "false",
                    "waterlogged": "false"
                }
            }
        ]
        "#;

        let json_properties = Value::from_str(PROPERTIES).unwrap();
        let json_states = Value::from_str(STATES).unwrap();

        if let Value::Array(states) = json_states {
            for state in states {
                let variants = Variant::from_json_props(&json_properties, state.get("properties").unwrap());

                assert_eq!(3, variants.len());
                assert_eq!(Variant::LeavesDistance(LeavesDistance(1)), variants[0]);

                let id = if let Value::Number(num) = state.get("id").unwrap() { num } else { panic!("Not a number") };

                match id.as_u64().unwrap() {
                    345 => {
                        assert_eq!(Variant::Persistent(Persistent(true)), variants[1]);
                        assert_eq!(Variant::Waterlogged(Waterlogged(true)), variants[2]);
                    },
                    346 => {
                        assert_eq!(Variant::Persistent(Persistent(true)), variants[1]);
                        assert_eq!(Variant::Waterlogged(Waterlogged(false)), variants[2]);
                    },
                    347 => {
                        assert_eq!(Variant::Persistent(Persistent(false)), variants[1]);
                        assert_eq!(Variant::Waterlogged(Waterlogged(true)), variants[2]);
                    },
                    348 => {
                        assert_eq!(Variant::Persistent(Persistent(false)), variants[1]);
                        assert_eq!(Variant::Waterlogged(Waterlogged(false)), variants[2]);
                    },
                    _ => {}
                }
            }
        }
    }
}
