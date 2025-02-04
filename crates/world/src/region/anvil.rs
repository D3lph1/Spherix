use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom};
use std::path::PathBuf;

use crate::region::pos::{ChunkWithinRegionPos, RegionPos};
use spherix_proto::io::Readable;

use crate::io::Compression;
use crate::region::RegionFile;

///
/// Region file structure:
///
/// <table>
///     <thead>
///         <tr>
///             <th>Byte</th>
///             <th>0 - 4095</th>
///             <th>4096-8191</th>
///             <th>8192-...</th>
///         </tr>
///     </thead>
///     <tbody>
///         <tr>
///             <td>Description</td>
///             <td>Locations (1024 entries)</td>
///             <td>Timestamps (1024 entries)</td>
///             <td>Chunks and unused space</td>
///         </tr>
///     </tbody>
/// </table>
///
///
/// Each location entry structure:
/// <table>
///     <thead>
///         <tr>
///             <th>Byte</th>
///             <th>1</th>
///             <th>2</th>
///             <th>3</th>
///             <th>4</th>
///         </tr>
///     </thead>
///     <tbody>
///         <tr>
///             <td>Description</td>
///             <td colspan="3">Offset from start of the file in KiB</td>
///             <td>Size of chunk in sectors (1 sector == 4 KiB)</td>
///         </tr>
///     </tbody>
/// </table>
pub struct Anvil {
    file: BufReader<File>,
    size: u64,
    pos: RegionPos
}

impl Anvil {
    const SECTOR_SIZE_BYTES: u64 = 4096;

    pub fn new(file: File, pos: RegionPos) -> Self {
        let size = file.metadata().unwrap().len();

        Self {
            file: BufReader::new(file),
            size,
            pos
        }
    }

    pub fn at(region_pos: RegionPos, base_path: PathBuf) -> Self {
        Self::new(
            File::open(base_path.join(Self::filename(&region_pos))).map_err(|e| {
                println!("{}", e);

                return e
            }).unwrap(),
            region_pos
        )
    }

    pub fn filename(region_pos: &RegionPos) -> String {
        format!("r.{}.{}.mca", region_pos.x(), region_pos.z())
    }

    /// Calculate position of location entry in the file.
    fn location_offset(chunk_pos: ChunkWithinRegionPos) -> u64 {
        (4 * ((chunk_pos.x() & 31) + (chunk_pos.z() & 31) * 32)) as u64
    }

    /// Reads 3 high bytes of location entry. The resulting value will be offset.
    fn offset_from_location_value(location: u64) -> u64 {
        let b1 = (location & 0xFF000000) >> 24;
        let b2 = (location & 0x00FF0000) >> 16;
        let b3 = (location & 0x0000FF00) >> 8;

        (b3 & 0xFF) | ((b2 & 0xFF) << 8) | ((b1 & 0x0F) << 16)
    }

    /// Reads 1 low bytes of location entry. The resulting value will be sectors count.
    fn sectors_from_location_value(location: u64) -> u64 {
        location & 0x000000FF
    }

    pub fn pos(&self) -> &RegionPos {
        &self.pos
    }
}

impl RegionFile for Anvil {
    fn read(&mut self, chunk_pos: ChunkWithinRegionPos) -> Option<nbt::Blob> {
        // Go to necessary location entry.
        let new_pos = self.file.seek(SeekFrom::Start(Self::location_offset(chunk_pos))).unwrap();

        if new_pos > self.size {
            return None
        }

        // Read location entry.
        let location = i32::read(&mut self.file).unwrap() as u64;
        let offset = Self::offset_from_location_value(location);
        let sectors = Self::sectors_from_location_value(location);
        if offset == 0 && sectors == 0 {
            return None
        }

        // Go to necessary chunk section.
        self.file.seek(SeekFrom::Start(Self::SECTOR_SIZE_BYTES * offset)).unwrap();
        // Skip length.
        let _ = i32::read(&mut self.file);
        let compression_type = u8::read(&mut self.file).unwrap();
        let compression = Compression::from(compression_type);

        // Read NBT with chunk data
        let buf = compression.decode(&mut self.file);
        let level = nbt::from_reader(&mut buf.as_slice()).unwrap();

        Some(level)
    }

    fn does_chunk_exist(&mut self, chunk_pos: ChunkWithinRegionPos) -> bool {
        // Go to necessary location entry
        let new_pos = self.file.seek(SeekFrom::Start(Self::location_offset(chunk_pos))).unwrap();

        if new_pos > self.size {
            return false
        }

        // Read location entry.
        let location = i32::read(&mut self.file).unwrap() as u64;
        let offset = Self::offset_from_location_value(location);
        let sectors = Self::sectors_from_location_value(location);

        !(offset == 0 && sectors == 0)
    }
}
