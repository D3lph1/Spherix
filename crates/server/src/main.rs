extern crate core;

use bevy_app::App;
use std::any::TypeId;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::net::{SocketAddr, SocketAddrV4};
use std::ops::Deref;
use std::path::PathBuf;
use std::process::exit;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use bevy_ecs::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use tokio::select;
use tokio::time::{sleep, Instant};
use tokio_util::sync::CancellationToken;
use tracing::subscriber::with_default;
use tracing::{error, info};

use spherix_config::build_config_from_env;
use spherix_log::{configure_logger, configure_temporary_logger};
use spherix_net::server::NetServer;
use spherix_proto::io::{VarInt, VarLong, Writable};
use spherix_world::block::block::Block;
use spherix_world::dimension::DimensionKind;

use crate::console::msg::CommandReceiver;
use crate::console::Console;
use crate::entities::living::player::{ChunkDataSentEvent, ChunkDidLoadedEvent, LoadPropertiesTask, PlayerNeedChunksEvent, PlayerSpawnedEvent, PlayerUnloadChunksEvent};
use crate::entities::UuidIdMap;
use crate::game::game::Game;
use crate::perf::worker::{DynamicTaskHandler, DynamicTaskHandlerDelegate, DynamicWorker};
use crate::perf::GeneralPurposeTaskSender;
use crate::plugin::CorePlugin;
use crate::server::{ClientReceiver, Server};
use crate::systems::command::{on_chat_command_packet, on_command, poll_commands, ChatCommandEvent};
use crate::systems::interaction::on_swing_hand;
use crate::systems::join::{despawn_player, handle_disconnect, on_join, poll_properties};
use crate::systems::keep_alive::{keep_alive, on_keep_alive_packet};
use crate::systems::message::{on_chat_message_packet, on_player_session};
use crate::systems::movement::{on_position_change, on_set_player_position_and_rotation_packet, on_set_player_position_packet, on_set_player_rotation_packet};
use crate::systems::packet::{ChatCommandPacketEvent, ChatMessagePacketEvent, KeepAlivePacketEvent, PlayerSessionPacketEvent, SetPlayerPositionAndRotationPacketEvent, SetPlayerPositionPacketEvent, SetPlayerRotationPacketEvent, SwingArmPacketEvent};
use crate::systems::player::{on_spawn, poll_packets, spawn_player_entities};
use crate::ticker::TickerPlugin;
use crate::world::dimension::{last_sent_set_center_chunk, load_chunks, on_chunk_data_sent, on_load_event, on_player_movement, poll_chunks, poll_unload_chunks_events};
use crate::world::player::worker::{LoadPropertiesTaskHandler, LoadPropertiesTaskResultReceiver};
use crate::world::world::World;
use spherix_math::vector::{Vector3, Vector3f};
use spherix_world::chunk::biome::Biome;
use spherix_world::chunk::palette::{create_biome_global_palette_from_json, create_block_global_palette_from_json};
use spherix_world::chunk::pos::ChunkPos;
use spherix_world::io::Compression;
use spherix_worldgen::biome::climate::json::create_biome_index_from_json;
use spherix_worldgen::biome::climate::point::ClimatePoint;
use spherix_worldgen::chunk::generator::NoiseBasedChunkGenerator;
use spherix_worldgen::noise::blending::blender::Blender;
use spherix_worldgen::noise::density::debug::DebugDensityFunction;
use spherix_worldgen::noise::density::density::{ChainMapper, DebugMapper, DensityFunction, DensityFunctionContext, DensityFunctions, InterpolatedCollector, Mapper, SetupNoiseMapper};
use spherix_worldgen::noise::density::noise::{NoiseDensityFunction, NoiseHolder};
use spherix_worldgen::noise::json::resolvable::Resolvable;
use spherix_worldgen::noise::json::value_resolver::{CachedValueResolver, CascadeValueResolver, FilesystemValueResolver, NoReturnValueResolver, ValueResolver};
use spherix_worldgen::noise::json::{deserializers_with_markers, Resolver};
use spherix_worldgen::noise::perlin::octave::MultiOctaveNoiseFactory;
use spherix_worldgen::noise::perlin::{DefaultNoise, LegacyNoise};
use spherix_worldgen::noise::settings::NoiseSettings;
use spherix_worldgen::rng::{Rng, RngForkable, XoroShiro};
use spherix_worldgen::surface::json::{condition_deserializers, rule_deserializers};

pub mod server;
pub mod game;
pub mod console;
pub mod world;
pub mod ticker;
pub mod player;
pub mod systems;
pub mod entities;
mod perf;
mod plugin;

const BANNER: &str = r"
>  ____            __
> /\  _`\         /\ \                     __
> \ \,\L\_\  _____\ \ \___      __   _ __ /\_\   __  _
>  \/_\__ \ /\ '__`\ \  _ `\  /'__`\/\`'__\/\ \ /\ \/'\
>    /\ \L\ \ \ \L\ \ \ \ \ \/\  __/\ \ \/ \ \ \\/>  </
>    \ `\____\ \ ,__/\ \_\ \_\ \____\\ \_\  \ \_\/\_/\_\
>     \/_____/\ \ \/  \/_/\/_/\/____/ \/_/   \/_/\//\/_/
>              \ \_\
>               \/_/                            v. 0.1.0
>
> Spherix: Blazingly fast Minecraft server written in Rust
>
> https://github.com/D3lph1/Spherix";

#[tokio::main]
async fn main() {
    let tmp_logger = configure_temporary_logger();

    let config = build_config_from_env();
    if config.is_err() {
        with_default(tmp_logger, || {
            error!("Error reading configuration file: {}", config.err().unwrap());
            error!("Launch interrupted by error");
        });

        exit(1);
    }

    drop(tmp_logger);

    let config = config.unwrap().unwrap();

    let _guard = configure_logger(&config.log);

    info!("{}", BANNER);

    if config.auth.enabled {
        let chat = if config.chat.secure {
            owo_colors::OwoColorize::green(&"with secure").to_string()
        } else {
            owo_colors::OwoColorize::yellow(&"without secure").to_string()
        };

        info!(
            "The server will be started in {} mode {} chat",
            owo_colors::OwoColorize::green(&"online"),
            chat
        );
    } else {
        info!("The server will be started in {} mode", owo_colors::OwoColorize::red(&"offline"));
    }

    ///

    // let mut xoro = XoroShiro::new(1);
    // let noise = DefaultNoise::create(
    //     &mut xoro,
    //     &vec![1.0, 1.0, 1.0, 1.0],
    //     -6
    // );
    //
    // let start = Instant::now();
    // let mut sum: i32 = 0;
    // for i in 0..10000 {
    //     sum = sum.wrapping_add(noise.sample(&Vector3f::new(17.0, i as f64, -81.0), 0.0, 0.0) as i32);
    // }
    //
    // println!("{}", sum);
    // println!("{:?}", start.elapsed());

    ///

    let w = config.world.path.inner();

    let path = "generated/reports/blocks.json";
    let f = std::fs::read_to_string(path).unwrap();
    let json: Value = serde_json::from_str(&f).unwrap();

    let now = Instant::now();

    let palette = Arc::new(create_block_global_palette_from_json(json));

    let _s = palette.get_objs_by_index(&Block::STONE).unwrap();
    // println!("{:?}", _s[0]);

    info!(
        "{} block states were successfully loaded from resource {} {}",
        owo_colors::OwoColorize::blue(&palette.len()),
        owo_colors::OwoColorize::green(&format!("\"{}\"", path)),
        owo_colors::OwoColorize::bright_black(&format!("({:.0?} elapsed)", now.elapsed()))
    );

    /// <>

    let now = Instant::now();

    let condition_resolver = Resolver::new(
        condition_deserializers(),
        Box::new(
            FilesystemValueResolver::new(
                PathBuf::from("./generated/data/minecraft/worldgen/noise")
            )
        )
    );

    // let mut surface_resolver = Resolver::new(
    //     rule_deserializers(condition_resolver, palette.clone()),
    //     Box::new(NoReturnValueResolver)
    // );
    // 
    // let path = "generated/data/minecraft/worldgen/noise_settings/overworld.json";
    // let file = File::open(PathBuf::from(path)).unwrap();
    // let json = serde_json::from_reader(file).unwrap();
    // let Value::Object(map) = json else {panic!()};
    // 
    // let surface_rule = map.get("surface_rule").unwrap();
    // 
    // let r = surface_resolver.resolve(surface_rule).unwrap();
    // 
    // info!(
    //     "Surface rule factory has been created from resource {} {}",
    //     owo_colors::OwoColorize::green(&format!("\"{}\"", path)),
    //     owo_colors::OwoColorize::bright_black(&format!("({:.0?} elapsed)", now.elapsed()))
    // );

    let v = FilesystemValueResolver::new(
        PathBuf::from("./generated/data/minecraft/worldgen/noise")
    );

    let condition_resolver = Resolver::new(
        condition_deserializers(),
        Box::new(
            FilesystemValueResolver::new(
                PathBuf::from("./generated/data/minecraft/worldgen/noise")
            )
        )
    );
    
    condition_resolver.contextual_name.set(Some("minecraft:clay_bands_offset".to_owned()));
    
    let a = NoiseHolder::resolve(
        &condition_resolver.resolve_value("minecraft:clay_bands_offset".to_owned()).unwrap(),
        &condition_resolver
    );

    // let r = df_resolver.resolve(&json).unwrap();

    // let Value::Object(root_obj) = json else {panic!()};
    // let noise_router = root_obj.get("noise_router").unwrap();
    // let Value::Object(final_density) = noise_router else {panic!()};
    // let final_density = final_density.get("vein_gap").unwrap();
    //
    // // let now = Instant::now();
    // let r = df_resolver.resolve(&final_density).unwrap();

    let mut rng = XoroShiro::new(1);

    let forked = rng.fork_pos();

    // println!("{:?}", now.elapsed());

    // let now = Instant::now();
    // let mut ctx = &mut DensityFunctionContext::default();
    // ctx.interpolation_counter = 1;
    // let s = rr.sample(&Vector3::new(2, 190, 10), &mut ctx);
    // // println!("{:?}", now.elapsed());
    // println!("{}", ctx.debug_tree);

    /// </>

    // let gen = NoiseBasedChunkGenerator::new(
    //     noise_settings,
    //     palette.clone(),
    //     forked
    // );
    //
    // gen.do_fill(
    //     ChunkPos::new(7, 11),
    //     Blender::new(HashMap::new(), HashMap::new()),
    //     -8,
    //     48
    // );

    let path = "generated/registry_codec.json";
    let f = std::fs::read_to_string(path).unwrap();
    let json: Value = serde_json::from_str(&f).unwrap();

    let biomes = json
        .as_object()
        .unwrap()
        .get("minecraft:worldgen/biome")
        .unwrap()
        .as_object()
        .unwrap()
        .get("value")
        .unwrap();

    let now = Instant::now();

    let biome_palette = create_biome_global_palette_from_json(biomes);
    let biomes_nbt = Biome::convert_array_to_nbt(biome_palette.all());

    info!(
        "{} biomes were successfully loaded from resource {} {}",
        owo_colors::OwoColorize::blue(&biome_palette.len()),
        owo_colors::OwoColorize::green(&format!("\"{}\"", path)),
        owo_colors::OwoColorize::bright_black(&format!("({:.0?} elapsed)", now.elapsed()))
    );

    let now = Instant::now();

    let mut world_mc = World::new(PathBuf::from("world"), palette, Arc::new(biome_palette));
    let overworld = world_mc.dimension_mut(DimensionKind::Overworld);

    // println!("{:?}", overworld.block_at(Vector3::new(0, 0, 97)));

    // println!("({:.2?} elapsed)", now.elapsed());
    // println!("{}", overworld.chunks_loaded());

    let cancel = CancellationToken::new();
    let console_cancel = cancel.clone();

    let (commands_tx, commands_rx) = flume::bounded(8);

    let (new_players_tx, new_players) = flume::bounded(8);
    let new_players = ClientReceiver(new_players);

    let s = Server::start(
        NetServer::new(
            SocketAddr::V4(SocketAddrV4::from_str(&format!("127.0.0.1:{}", config.network.port)).unwrap()),
            cancel.clone(),
            new_players_tx,
            biomes_nbt,
            config.clone(),
        )
    ).await;

    let console = Console::new(commands_tx, console_cancel);

    tokio::spawn({
        async move {
            console.serve();

            info!("Shutting down console worker...")
        }
    });

    let (task_tx, task_rx) = flume::unbounded();
    let (res_tx, res_rx) = flume::unbounded();

    let delegated: Box<dyn DynamicTaskHandler + Send + Sync> = Box::new(LoadPropertiesTaskHandler {
        config: config.clone(),
        result_tx: res_tx,
    });

    let worker = DynamicWorker::new(
        DynamicTaskHandlerDelegate(
            HashMap::from([(
                TypeId::of::<LoadPropertiesTask>(),
                delegated
            )])
        ),
        task_rx,
        4,
    );

    std::thread::spawn(|| worker.run());

    std::thread::spawn(move || {
        let mut app = App::new();

        app.add_plugins(CorePlugin);
        app.add_plugins(TickerPlugin::new(20));

        app.insert_resource(world_mc);
        app.insert_resource(s);
        app.insert_resource(Game::new());
        app.insert_resource(new_players);
        app.insert_resource(config);
        app.insert_resource(CommandReceiver(commands_rx));

        //

        app.insert_resource(GeneralPurposeTaskSender(task_tx));
        app.insert_resource(LoadPropertiesTaskResultReceiver(res_rx));

        app.run();
    });

    loop {
        select! {
            _ = cancel.cancelled() => {
                return;
            },
            _ = sleep(Duration::from_millis(50)) => {}
        }
    }
}
