use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::thread;

use bevy_ecs::event::EventReader;
use bevy_ecs::prelude::{Changed, Commands, Component, Entity, EventWriter, Res, Without};
use bevy_ecs::query::With;
use bevy_ecs::system::Query;
use flume::{unbounded, Receiver, Sender};
use gxhash::GxBuildHasher;

use spherix_config::Config;
use spherix_math::vector::{OrderedSquareIter, RadialIter};
use spherix_proto::io::VarInt;
use spherix_proto::packet::clientbound::{PlayMapping, SetCenterChunk, SetDefaultSpawnPosition, UnloadChunk};
use spherix_world::chunk::pos::{ChunkPos, GlobalChunkPos};
use spherix_world::dimension::DimensionKind;

use crate::entities::living::player::{ChunkDidLoadedEvent, DimensionKindPosPair, KnownChunks, LastSentSetCenterChunkPacket, LoadedChunksCounter, PlayerNeedChunksEvent, PlayerType, PlayerUnloadChunksEvent, Spawned, ToSend};
use crate::entities::Uuid;
use crate::perf::worker::{ForceSend, StaticWorker};
use crate::player::Position;
use crate::world::region::generator::RegionGeneratorWorkerHandler;
use crate::world::region::worker::{ChunkTask, LoadChunkTask};
use crate::world::world::World;
use spherix_world::chunk::palette::{BiomeGlobalPalette, BlockGlobalPalette};
use spherix_worldgen::chunk::column::ChunkColumn;

pub mod layout;

pub struct Chunks(pub RwLock<HashMap<ChunkPos, Option<Arc<ChunkColumn>>>>);

pub struct Dimension {
    dim: DimensionKind,
    dir: PathBuf,
    palette: Arc<BlockGlobalPalette>,
    biomes_palette: Arc<BiomeGlobalPalette>,

    chunk_tasks_tx: Sender<ChunkTask>,
    chunk_rx: Receiver<Arc<ChunkColumn>>,

    chunks: Chunks,

    player_chunks: Arc<RwLock<HashMap<ChunkPos, HashMap<Entity, ()>>>>,
}

impl Dimension {
    pub fn new(dim: DimensionKind, dir: PathBuf, palette: Arc<BlockGlobalPalette>, biomes_palette: Arc<BiomeGlobalPalette>) -> Self {
        let (chunk_tasks_tx, chunk_tasks_rx) = unbounded();
        let (chunk_tx, chunk_rx) = unbounded();
        // let worker = StaticWorker::new(
        //     RegionLoadWorkerHandler::new(
        //         dir.clone(),
        //         palette.clone(),
        //         biomes_palette.clone(),
        //         chunk_tx,
        //     ),
        //     chunk_tasks_rx,
        //     4,
        // );

        let generator_chunk_cache = Arc::new(RwLock::new(HashMap::with_hasher(GxBuildHasher::default())));

        let (handler, noise_settings, entropy_bag, rule_factory) = RegionGeneratorWorkerHandler::new(
            palette.clone(),
            biomes_palette.clone(),
            generator_chunk_cache,
            chunk_tx,
        );

        let worker = StaticWorker::new(
            handler,
            (unsafe { ForceSend::new(noise_settings) }, entropy_bag, rule_factory),
            chunk_tasks_rx,
            4,
        );

        thread::spawn(|| worker.run());

        Self {
            dim,
            dir,
            palette,
            biomes_palette,
            // worker,
            chunk_tasks_tx,
            chunk_rx,
            chunks: Chunks(Default::default()),
            player_chunks: Arc::default(),
        }
    }

    fn insert_to_player_chunks(&self, pos: ChunkPos, player: Entity) {
        let mut guard = self.player_chunks.write().unwrap();
        if !guard.contains_key(&pos) {
            guard.insert(pos.clone(), HashMap::new());
        }

        let players = guard.get_mut(&pos).unwrap();
        players.insert(player, ());
        // drop the guard as early as possible
        drop(guard);
    }

    pub fn submit_chunk_task(&self, task: ChunkTask) {
        self.chunk_tasks_tx.send(task).unwrap();
    }
}


pub fn load_chunks(
    world: Res<World>,
    mut query: Query<
        (&Uuid, &mut KnownChunks, &mut LoadedChunksCounter, &ToSend),
        (With<PlayerType>)
    >,
    mut rx: EventReader<PlayerNeedChunksEvent>,
) {
    for s in rx.read() {
        for chunk in s.chunks.clone() {
            let dim = world.dimension(chunk.dim.clone());

            let mut guard = dim.chunks.0.write().unwrap();

            if guard.contains_key(&chunk.vec) && guard.get(&chunk.vec).unwrap().is_some() {
                dim.insert_to_player_chunks(chunk.vec.clone(), s.entity);

                // Drop guard to prevent deadlock in send_chunk()
                drop(guard);

                let (_, mut known_chunks, mut counter, to_send) = query.get_mut(s.entity).unwrap();
                send_chunk(chunk.vec.clone(), dim, to_send);
                if known_chunks.0.contains_key(&chunk.vec) {
                    let known_chunk = known_chunks.0.get_mut(&chunk.vec).unwrap();
                    *known_chunk = true;
                } else {
                    known_chunks.0.insert(chunk.vec, true);
                }

                counter.0 += 1;
                continue;
            }

            guard.insert(chunk.vec.clone(), None);
            // drop the guard as early as possible
            drop(guard);

            dim.insert_to_player_chunks(chunk.vec.clone(), s.entity);
            dim.submit_chunk_task(ChunkTask::Load(LoadChunkTask(chunk.vec)));
        }
    }
}

pub fn poll_chunks(
    world: Res<World>,
    mut tx: EventWriter<ChunkDidLoadedEvent>,
) {
    for (_, dim) in world.dimensions() {
        for chunk in dim.chunk_rx.try_iter() {
            let pos = chunk.pos();
            let mut guard_chunk_col = dim.chunks.0.write().unwrap();

            if guard_chunk_col.contains_key(&chunk.pos()) {
                guard_chunk_col.insert(chunk.pos(), Some(chunk));
            }

            drop(guard_chunk_col);

            tx.send(ChunkDidLoadedEvent {
                chunk: GlobalChunkPos { vec: pos, dim: dim.dim.clone() },
            });
        }
    }
}

pub fn on_load_event(
    world: Res<World>,
    mut query: Query<
        (&Uuid, &mut KnownChunks, &mut LoadedChunksCounter, &ToSend),
        (With<PlayerType>)
    >,
    mut rx: EventReader<ChunkDidLoadedEvent>,
) {
    for event in rx.read() {
        let dim = world.dimension(event.chunk.dim.clone());

        let guard = dim.player_chunks.read().unwrap();
        if !guard.contains_key(&event.chunk.vec) {
            continue;
        }

        let players = guard.get(&event.chunk.vec).unwrap();

        for (player, _) in players {
            let r = query.get_mut(player.clone());
            if r.is_err() {
                continue;
            }

            let (_, mut known_chunks, mut counter, to_send) = r.unwrap();
            if !known_chunks.contains_key(&event.chunk.vec) {
                continue;
            }

            send_chunk(event.chunk.vec.clone(), dim, to_send);
            let val = known_chunks.get_mut(&event.chunk.vec).unwrap();
            *val = true;
            counter.0 += 1;
        }

        //
    }
}

fn send_chunk(pos: ChunkPos, dim: &Dimension, to_send: &ToSend) {
    let guard_chunk_col = dim.chunks.0.read().unwrap();
    let entry = guard_chunk_col.get(&pos).unwrap();
    let column = entry.as_ref().unwrap();

    if to_send.send(PlayMapping::ChunkData(column.inner().to_load_packet())).is_err() {
        // If receiving side of the channel was dropped, just return
        return;
    }
}

pub fn on_chunk_data_sent(
    query: Query<(Entity, &LoadedChunksCounter, &ToSend), (With<PlayerType>, Without<Spawned>, Changed<LoadedChunksCounter>)>,
    mut commands: Commands
) {
    for (entity, counter, to_send) in query.iter() {

        println!("COUNTER: {}", counter.0);

        if counter.0 >= 30 {
            // This packet is very important. It is required to hide "Loading terrain" screen.
            // SynchronizePlayerPosition packet does not affect this screen anyhow (despite the
            // fact that the documentation says so).
            to_send.send(PlayMapping::SetDefaultSpawnPosition(SetDefaultSpawnPosition {
                location: spherix_proto::io::Position::new(20, 70, -10),
                angle: 0.0,
            })).unwrap();

            commands
                .entity(entity)
                .insert(Spawned);
        }
    }
}

pub fn on_player_movement(
    config: Res<Config>,
    query: Query<(Entity, &Uuid, &Position, &KnownChunks), (With<PlayerType>, With<Spawned>, Changed<Position>)>,
    mut commands: Commands,
    mut tx: EventWriter<PlayerNeedChunksEvent>,
    mut tx_unload: EventWriter<PlayerUnloadChunksEvent>,
) {
    for (entity, _, pos, known_chunks) in query.iter() {
        let chunk_pos: ChunkPos = pos.clone().into();

        let new_chunks_iter =
            RadialIter::new(
                OrderedSquareIter::new(
                    GlobalChunkPos::new(DimensionKind::Overworld, chunk_pos.clone()),
                    config.world.view_distance as usize,
                )
            );

        let mut new_known_chunks = KnownChunks::default();
        let mut chunks_to_send = Vec::new();

        for chunk in new_chunks_iter {
            let chunk1 = chunk.clone().vec;
            if !known_chunks.contains_key(&chunk1) {
                chunks_to_send.push(chunk);
                new_known_chunks.insert(chunk1, false);
            } else {
                new_known_chunks.insert(chunk1.clone(), *known_chunks.get(&chunk1).unwrap());
            }
        }

        let mut chunks_to_unload = Vec::new();
        for (old_chunk, _) in &known_chunks.0 {
            if !new_known_chunks.contains_key(old_chunk) {
                chunks_to_unload.push(GlobalChunkPos::new(DimensionKind::Overworld, old_chunk.clone()));
            }
        }

        if chunks_to_send.len() > 0 || chunks_to_unload.len() > 0 {
            commands.entity(entity.clone())
                .insert(new_known_chunks);

            if chunks_to_send.len() > 0 {
                tx.send(
                    PlayerNeedChunksEvent {
                        entity: entity.clone(),
                        chunks: Box::new(chunks_to_send.into_iter()),
                    }
                );
            }

            if chunks_to_unload.len() > 0 {
                tx_unload.send(PlayerUnloadChunksEvent {
                    entity: entity.clone(),
                    chunks: chunks_to_unload,
                });
            }
        }
    }
}

/// As stated [`here`]:
///
/// Updates the client's location. This is used to determine what
/// chunks should remain loaded and if a chunk load should be ignored; chunks
/// outside of the view distance may be unloaded.
///
/// Sent whenever the player moves across a chunk border horizontally, and also
/// (according to testing) for any integer change in the vertical axis, even if
/// it doesn't go across a chunk section border.
///
/// [`here`]: https://wiki.vg/index.php?title=Protocol&oldid=18242#Set_Center_Chunk
pub fn last_sent_set_center_chunk(
    query: Query<(
        Entity,
        &Position,
        &DimensionKind,
        &LastSentSetCenterChunkPacket,
        &ToSend
    ), (
        With<PlayerType>, With<Spawned>,
        Changed<Position>
    )>,
    mut commands: Commands,
) {
    for (
        entity,
        player_pos,
        player_dim,
        last_sent_set_center_chunk_packet,
        to_send
    ) in query.iter() {
        let chunk_pos: ChunkPos = player_pos.clone().into();

        if last_sent_set_center_chunk_packet.0.is_some() {
            let prev_dim_pos_pair = last_sent_set_center_chunk_packet.0.as_ref().unwrap();
            let prev_chunk_pos: ChunkPos = prev_dim_pos_pair.pos.clone().into();

            if prev_dim_pos_pair.dim != *player_dim
                || (prev_dim_pos_pair.dim == *player_dim && prev_chunk_pos != chunk_pos)
                // any integer change in the vertical axis
                || (prev_dim_pos_pair.pos.y as i32 != player_pos.y as i32)
            {
                send_set_center_chunk_packet(
                    &mut commands,
                    to_send,
                    entity.clone(),
                    *player_dim,
                    player_pos.clone(),
                    GlobalChunkPos::new(*player_dim, chunk_pos),
                );
            }
        } else {
            send_set_center_chunk_packet(
                &mut commands,
                to_send,
                entity.clone(),
                *player_dim,
                player_pos.clone(),
                GlobalChunkPos::new(*player_dim, chunk_pos),
            );
        }
    }
}

fn send_set_center_chunk_packet(
    commands: &mut Commands,
    to_send: &ToSend,
    entity: Entity,
    player_dim: DimensionKind,
    player_pos: Position,
    chunk_pos: GlobalChunkPos,
) {
    to_send.send(PlayMapping::SetCenterChunk(SetCenterChunk {
        chunk_x: VarInt(chunk_pos.x()),
        chunk_z: VarInt(chunk_pos.z()),
    })).unwrap();

    commands
        .entity(entity)
        .insert(LastSentSetCenterChunkPacket(Some(DimensionKindPosPair {
            dim: player_dim,
            pos: player_pos,
        })));
}

pub fn poll_unload_chunks_events(
    mut rx: EventReader<PlayerUnloadChunksEvent>,
    query: Query<&ToSend, (With<PlayerType>, With<Spawned>)>
) {
    for event in rx.read() {
        let to_send = query.get(event.entity).unwrap();

        for chunk in &event.chunks {
            to_send.send(
                PlayMapping::UnloadChunk(UnloadChunk {
                    x: chunk.x(),
                    z: chunk.z(),
                })
            ).unwrap();
        }
    }
}
