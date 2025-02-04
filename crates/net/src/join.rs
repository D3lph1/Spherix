use std::collections::HashMap;

use flume::{Receiver, Sender};
use nbt::{Blob, Value};
use uuid::Uuid;

use spherix_proto::io::VarInt;
use spherix_proto::packet::clientbound::PlayMapping as ClientboundPlayMapping;
use spherix_proto::packet::clientbound::{Login, PlayMapping};
use spherix_proto::packet::serverbound::PlayMapping as ServerboundPlayMapping;
use spherix_registry::damage::damage_types;

use crate::client::Client;
use crate::conn::Connection;
use crate::io::{Reader, Writer};

pub struct JoinContext<'a> {
    pub name: String,
    pub uuid: Uuid,
    pub conn: &'a mut Connection,
    pub reader: &'a mut Reader,
    pub writer: &'a mut Writer,
    biomes: Value,
}

impl<'a> JoinContext<'a> {
    pub fn new(
        name: String,
        uuid: Uuid,
        conn: &'a mut Connection,
        reader: &'a mut Reader,
        writer: &'a mut Writer,
        biomes: Value,
    ) -> Self {
        Self {
            name,
            uuid,
            conn,
            reader,
            writer,
            biomes,
        }
    }

    pub fn to_client(&self, received: Receiver<ServerboundPlayMapping>, to_send: Sender<ClientboundPlayMapping>) -> Client {
        Client::new(self.name.clone(), self.uuid, received, to_send)
    }
}

pub async fn join(ctx: JoinContext<'_>) {
    let mut dimension_type = HashMap::new();

    {
        dimension_type.insert("type".to_owned(), "minecraft:dimension_type".into());
        dimension_type.insert("value".to_owned(), Value::List(vec![
            Value::Compound(
                HashMap::from([
                    ("name".to_owned(), "minecraft:overworld".into()),
                    ("id".to_owned(), Value::Int(0)),
                    (
                        "element".to_owned(),
                        Value::Compound(
                            HashMap::from([
                                ("has_raids".to_owned(), Value::Byte(1)),
                                ("monster_spawn_block_light_limit".to_owned(), Value::Int(10)), // Val just for example
                                ("monster_spawn_light_level".to_owned(), Value::Int(2)), // Val just for example
                                ("piglin_safe".to_owned(), Value::Byte(0)),
                                ("ambient_light".to_owned(), Value::Float(0.0)),
                                ("infiniburn".to_owned(), "#minecraft:infiniburn_overworld".into()),
                                ("logical_height".to_owned(), Value::Int(384)),
                                ("height".to_owned(), Value::Int(384)),
                                ("min_y".to_owned(), Value::Int(-64)),
                                ("respawn_anchor_works".to_owned(), Value::Byte(0)),
                                ("bed_works".to_owned(), Value::Byte(1)),
                                ("coordinate_scale".to_owned(), Value::Double(1.0)),
                                ("natural".to_owned(), Value::Byte(1)),
                                ("ultrawarm".to_owned(), Value::Byte(0)),
                                ("has_ceiling".to_owned(), Value::Byte(0)),
                                ("has_skylight".to_owned(), Value::Byte(1))
                            ])
                        )
                    )
                ])
            ),
            // Value::Compound(
            //     HashMap::from([
            //         ("name".to_owned(), "minecraft:overworld_caves".into()),
            //         ("id".to_owned(), Value::Int(1)),
            //         (
            //             "element".to_owned(),
            //             Value::Compound(
            //                 HashMap::from([
            //                     ("ambient_light".to_owned(), Value::Float(0.0)),
            //                     ("bed_works".to_owned(), Value::Byte(1)),
            //                     ("coordinate_scale".to_owned(), Value::Double(1.0)),
            //                     ("effects".to_owned(), "minecraft:overworld".into()),
            //                     ("has_ceiling".to_owned(), Value::Byte(1)),
            //                     ("has_raids".to_owned(), Value::Byte(1)),
            //                     ("has_skylight".to_owned(), Value::Byte(1)),
            //                     ("infiniburn".to_owned(), "minecraft:infiniburn_overworld".into()),
            //                     ("height".to_owned(), Value::Int(256)),
            //                     ("logical_height".to_owned(), Value::Int(256)),
            //                     ("min_y".to_owned(), Value::Int(0)),
            //                     ("natural".to_owned(), Value::Byte(1)),
            //                     ("piglin_safe".to_owned(), Value::Byte(0)),
            //                     ("respawn_anchor_works".to_owned(), Value::Byte(0)),
            //                     ("ultrawarm".to_owned(), Value::Byte(0)),
            //                     ("monster_spawn_light_level".to_owned(), Value::Int(2)), // Val just for example
            //                     ("monster_spawn_block_light_limit".to_owned(), Value::Int(10)), // Val just for example
            //                 ])
            //             )
            //         )
            //     ])
            // ),
            // Value::Compound(
            //     HashMap::from([
            //         ("name".to_owned(), "minecraft:the_nether".into()),
            //         ("id".to_owned(), Value::Int(2)),
            //         (
            //             "element".to_owned(),
            //             Value::Compound(
            //                 HashMap::from([
            //                     ("ambient_light".to_owned(), Value::Float(0.10000000149011612)),
            //                     ("bed_works".to_owned(), Value::Byte(0)),
            //                     ("coordinate_scale".to_owned(), Value::Double(8.0)),
            //                     ("effects".to_owned(), "minecraft:the_nether".into()),
            //                     ("fixed_time".to_owned(), Value::Short(18000)),
            //                     ("has_ceiling".to_owned(), Value::Byte(1)),
            //                     ("has_raids".to_owned(), Value::Byte(0)),
            //                     ("has_skylight".to_owned(), Value::Byte(0)),
            //                     ("infiniburn".to_owned(), "minecraft:infiniburn_nether".into()),
            //                     ("height".to_owned(), Value::Int(128)),
            //                     ("logical_height".to_owned(), Value::Int(128)),
            //                     ("min_y".to_owned(), Value::Int(0)),
            //                     ("natural".to_owned(), Value::Byte(0)),
            //                     ("piglin_safe".to_owned(), Value::Byte(1)),
            //                     ("respawn_anchor_works".to_owned(), Value::Byte(1)),
            //                     ("ultrawarm".to_owned(), Value::Byte(1)),
            //                     ("monster_spawn_light_level".to_owned(), Value::Int(2)), // Val just for example
            //                     ("monster_spawn_block_light_limit".to_owned(), Value::Int(10)), // Val just for example
            //                 ])
            //             )
            //         )
            //     ])
            // ),
            // Value::Compound(
            //     HashMap::from([
            //         ("name".to_owned(), "minecraft:the_end".into()),
            //         ("id".to_owned(), Value::Int(3)),
            //         (
            //             "element".to_owned(),
            //             Value::Compound(
            //                 HashMap::from([
            //                     ("ambient_light".to_owned(), Value::Float(0.0)),
            //                     ("bed_works".to_owned(), Value::Byte(0)),
            //                     ("coordinate_scale".to_owned(), Value::Double(1.0)),
            //                     ("effects".to_owned(), "minecraft:the_end".into()),
            //                     ("fixed_time".to_owned(), Value::Short(6000)),
            //                     ("has_ceiling".to_owned(), Value::Byte(0)),
            //                     ("has_raids".to_owned(), Value::Byte(1)),
            //                     ("has_skylight".to_owned(), Value::Byte(0)),
            //                     ("infiniburn".to_owned(), "minecraft:infiniburn_end".into()),
            //                     ("height".to_owned(), Value::Int(256)),
            //                     ("logical_height".to_owned(), Value::Int(256)),
            //                     ("min_y".to_owned(), Value::Int(0)),
            //                     ("natural".to_owned(), Value::Byte(0)),
            //                     ("piglin_safe".to_owned(), Value::Byte(0)),
            //                     ("respawn_anchor_works".to_owned(), Value::Byte(0)),
            //                     ("ultrawarm".to_owned(), Value::Byte(0)),
            //                     ("monster_spawn_light_level".to_owned(), Value::Int(2)), // Val just for example
            //                     ("monster_spawn_block_light_limit".to_owned(), Value::Int(10)), // Val just for example
            //                 ])
            //             )
            //         )
            //     ])
            // )
        ]));
    }

    let mut damage_type = HashMap::new();

    {
        damage_type.insert("type".to_owned(), "minecraft:damage_type".into());
        damage_type.insert("value".to_owned(), damage_types());
    }


    let mut worldgen_biome = HashMap::new();

    {
        worldgen_biome.insert("type".to_owned(), "minecraft:worldgen/biome".into());
        worldgen_biome.insert("value".to_owned(), ctx.biomes);
    }

    let mut chat_type = HashMap::new();

    {
        chat_type.insert("type".to_owned(), "minecraft:chat_type".into());
        chat_type.insert("value".to_owned(), Value::List(vec![
            Value::Compound(
                HashMap::from([
                    ("name".to_owned(), "minecraft:chat".into()),
                    ("id".to_owned(), Value::Int(0)),
                    (
                        "element".to_owned(),
                        Value::Compound(
                            HashMap::from([
                                (
                                    "chat".to_owned(),
                                    Value::Compound(
                                        HashMap::from([
                                            (
                                                "parameters".to_owned(),
                                                Value::List(vec![
                                                    Value::String("sender".to_owned()),
                                                    Value::String("content".to_owned()),
                                                ])
                                            ),
                                            (
                                                "translation_key".to_owned(),
                                                Value::String("chat.type.text".to_owned()),
                                            )
                                        ])
                                    )
                                ),
                                (
                                    "narration".to_owned(),
                                    Value::Compound(
                                        HashMap::from([
                                            (
                                                "parameters".to_owned(),
                                                Value::List(vec![
                                                    Value::String("sender".to_owned()),
                                                    Value::String("content".to_owned()),
                                                ])
                                            ),
                                            (
                                                "translation_key".to_owned(),
                                                Value::String("chat.type.text.narrate".to_owned()),
                                            )
                                        ])
                                    )
                                )
                            ])
                        )
                    )
                ])
            ),
        ]));
    }

    let mut blob = Blob::new();

    blob.insert("minecraft:dimension_type", Value::Compound(dimension_type)).unwrap();
    blob.insert("minecraft:damage_type", Value::Compound(damage_type)).unwrap();
    blob.insert("minecraft:worldgen/biome", Value::Compound(worldgen_biome)).unwrap();
    blob.insert("minecraft:chat_type", Value::Compound(chat_type)).unwrap();


    let packet = Login {
        entity_id: 1, // TODO: Change!
        is_hardcore: false,
        gamemode: 1,
        previous_gamemode: -1,
        dimensions: Box::new([
            "overworld".to_owned()
        ]),
        registry_codec: blob,
        dimension_type: "overworld".to_string(),
        dimension_name: "overworld".to_string(),
        hashed_seed: 0,
        max_players: VarInt(100),
        view_distance: VarInt(8),
        simulation_distance: VarInt(8),
        reduced_debug_info: false,
        enable_respawn_screen: false,
        is_debug: false, // Enables debug world, should be disabled
        is_flat: false,
        has_death_location: false,
    };

    ctx.writer.write(PlayMapping::Login(packet)).await.unwrap();

    // let packet = SetDefaultSpawnPosition {
    //     location: Position::new(20, 70, -10),
    //     angle: Angle(0),
    // };

    // ctx.writer.write(PlayMapping::SetDefaultSpawnPosition(packet)).await.unwrap();

    // let a = ctx.reader.read::<spherix_proto::packet::serverbound::PlayMapping>().await;
    //
    // println!("{:?}", a);
}
