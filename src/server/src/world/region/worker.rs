use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use flume::{Receiver, Sender};
use owo_colors::OwoColorize;
use tracing::trace;

use spherix_world::chunk::pos::ChunkPos;
use spherix_world::region::pos::{ChunkWithinRegionPos, RegionPos};

use crate::perf::worker::StaticTaskHandle;
use spherix_world::chunk::column::ChunkColumn;
use spherix_world::chunk::palette::{BiomeGlobalPalette, BlockGlobalPalette};
use spherix_world::region::anvil::Anvil;
use spherix_world::region::RegionFile;

pub struct LoadChunkTask(pub ChunkPos);

pub enum ChunkTask {
    Load(LoadChunkTask)
}

struct RegionDescriptor {
    region: Anvil,
    touched_at: Instant,
}

impl RegionDescriptor {
    const DESCRIPTOR_LIFETIME: Duration = Duration::from_secs(60);

    fn new(region: Anvil) -> Self {
        Self { region, touched_at: Instant::now() }
    }

    fn old_enough(&self) -> bool {
        self.touched_at.elapsed() > Self::DESCRIPTOR_LIFETIME
    }
}

impl Drop for RegionDescriptor {
    fn drop(&mut self) {
        trace!("Closing unused region descriptor for file {}", filename_fancy(&self.region.pos()))
    }
}

fn filename_fancy(region_pos: &RegionPos) -> String {
    let mut s = String::new();
    s.push('"');
    s.push_str(&Anvil::filename(region_pos));
    s.push('"');

    s.green().to_string()
}

pub struct RegionLoadWorkerHandler {
    dir: PathBuf,
    palette: Arc<BlockGlobalPalette>,
    biomes_palette: Arc<BiomeGlobalPalette>,

    regions: Mutex<HashMap<RegionPos, RegionDescriptor>>,
    chunk_tx: Sender<ChunkColumn>,
}

impl RegionLoadWorkerHandler {
    pub fn new(
        dir: PathBuf,
        palette: Arc<BlockGlobalPalette>,
        biomes_palette: Arc<BiomeGlobalPalette>,
        chunk_tx: Sender<ChunkColumn>
    ) -> Self {
        Self {
            dir,
            palette,
            biomes_palette,
            regions: Mutex::default(),
            chunk_tx,
        }
    }

    fn load(&self, pos: ChunkPos) {
        let region_pos: RegionPos = pos.clone().into();

        let mut regions_guard = self.regions.lock().unwrap();
        let region = if regions_guard.contains_key(&region_pos) {
            let region = regions_guard.get_mut(&region_pos).unwrap();
            region.touched_at = Instant::now();

            region
        } else {
            regions_guard.insert(
                region_pos.clone(),
                RegionDescriptor::new(Anvil::at(region_pos.clone(), self.dir.join("region"))),
            );
            trace!("Opening a region descriptor for file {}", filename_fancy(&region_pos));

            regions_guard.get_mut(&region_pos.clone()).unwrap()
        };

        let chunk_pos: ChunkWithinRegionPos = pos.clone().into();

        let now = Instant::now();
        let chunk_blob = region.region.read(chunk_pos.clone()).unwrap();
        let io_elapsed = now.elapsed();

        let now = Instant::now();
        let mut chunk = ChunkColumn::from_nbt(chunk_blob, self.palette.clone(), self.biomes_palette.clone());
        let handle_elapsed = now.elapsed();

        trace!(
            "Chunk ({}, {}) was loaded for the region ({}, {}) {}",
            pos.x().bright_red(),
            pos.z().bright_blue(),
            region_pos.x().bright_red(),
            region_pos.z().bright_blue(),
            format!("({:.0?} I/O + {:.0?} elapsed)", io_elapsed, handle_elapsed).bright_black()
        );

        self.chunk_tx.send(chunk).unwrap();
    }

    fn close_unused_descriptors(&self) {
        self.regions.lock().unwrap().retain(|_, descriptor| !descriptor.old_enough());
    }
}

impl StaticTaskHandle<ChunkTask, ()> for RegionLoadWorkerHandler {
    fn handle(&self, task: ChunkTask, _: ()) {
        match task {
            ChunkTask::Load(task) => self.load(task.0)
        }

        self.close_unused_descriptors();
    }
}
