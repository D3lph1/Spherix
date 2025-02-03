use std::collections::HashMap;

use nbt::Value;

macro_rules! damage_type {
    (
        $id:literal, $exhaustion:literal, $scaling:literal
    ) => {
        Value::Compound(
            HashMap::from([
                ("name".to_owned(), Value::String(format!("minecraft:{}", $id))),
                ("id".to_owned(), Value::Int(0)),
                (
                    "element".to_owned(),
                    Value::Compound(
                        HashMap::from([
                            ("exhaustion".to_owned(), Value::Float($exhaustion)),
                            ("message_id".to_owned(), Value::String($id.to_owned())),
                            ("scaling".to_owned(), Value::String($scaling.to_owned()))
                        ])
                    )
                )
            ])
        )
    }
}

pub fn damage_types() -> Value {
    Value::List(vec![
        damage_type!("in_fire", 0.1, "always"),
        damage_type!("lightning_bolt", 0.1, "always"),
        damage_type!("on_fire", 0.1, "always"),
        damage_type!("lava", 0.1, "always"),
        damage_type!("hot_floor", 0.1, "always"),
        damage_type!("in_wall", 0.1, "never"),
        damage_type!("cramming", 0.1, "always"),
        damage_type!("drown", 0.1, "never"),
        damage_type!("starve", 0.1, "never"),
        damage_type!("cactus", 0.1, "always"),
        damage_type!("fall", 0.1, "always"),
        damage_type!("fly_into_wall", 0.1, "always"),
        damage_type!("out_of_world", 0.1, "always"),
        damage_type!("generic", 0.1, "always"),
        damage_type!("magic", 0.1, "always"),
        damage_type!("wither", 0.1, "always"),
        damage_type!("dragon_breath", 0.1, "always"),
        damage_type!("dry_out", 0.1, "always"),
        damage_type!("sweet_berry_bush", 0.1, "always"),
        damage_type!("freeze", 0.1, "always"),
        damage_type!("stalagmite", 0.1, "always")
    ])
}
