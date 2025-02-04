use crate::block::packed::PackedArray;
use crate::chunk::palette::local::{GlobalLocalPalette, HashMapLocalPalette, LocalId, LocalPalette, LocalPalettes, PutStatus};

pub struct LocalPaletteResizer {
    pub min_fit_size: u8,
    pub max_fit_size: u8,
    pub threshold: u8
}

impl LocalPaletteResizer {
    pub fn new(min_fit_size: u8, max_fit_size: u8, threshold: u8) -> Self {
        Self {
            min_fit_size,
            max_fit_size,
            threshold,
        }
    }

    pub fn resize(&self, mut bits: u8, palette: &LocalPalettes, data: &PackedArray) -> (LocalPalettes, PackedArray) {
        if bits < self.min_fit_size {
            bits = self.min_fit_size;
        }

        let mut resized_palette: LocalPalettes = if bits < self.threshold {
            match palette {
                LocalPalettes::SingleValued(_) => {
                    LocalPalettes::HashMap(HashMapLocalPalette::with_capacity(bits, 2))
                },
                LocalPalettes::HashMap(_) => {
                    LocalPalettes::HashMap(HashMapLocalPalette::with_capacity(bits, palette.len() + 1))
                },
                _ => LocalPalettes::HashMap(HashMapLocalPalette::new(bits))
            }
        } else {
            bits = self.max_fit_size;
            LocalPalettes::Global(GlobalLocalPalette::new(self.max_fit_size))
        };

        let mut resized_data = PackedArray::with_capacity_for(bits as usize, data.len());

        for i in 0..data.len() {
            let global = palette.global_by_local(LocalId(data.get(i))).unwrap();
            let local = match resized_palette.local_by_global(global) {
                Some(local) => local,
                None => {
                    match resized_palette.put(global) {
                        PutStatus::Stored { local } => local,
                        PutStatus::NeedResize { .. } => unreachable!()
                    }
                }
            };

            resized_data.set(i, local.0);
        }

        (resized_palette, resized_data)
    }
}
