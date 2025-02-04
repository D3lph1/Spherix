pub mod density;
pub mod perlin;
pub mod math;
pub mod json;
pub mod blending;
pub mod router;
pub mod settings;

#[cfg(test)]
mod tests {
    use crate::noise::density::density::DensityFunction;
    use crate::noise::density::density::{DensityFunctionContext, SetupNoiseMapper};
    use crate::noise::json::value_resolver::{CascadeValueResolver, MockValueResolver};
    use crate::noise::json::{deserializers, Resolver};
    use crate::rng::{RngForkable, XoroShiro};
    use serde_json::Value;
    use spherix_math::vector::Vector3;
    use spherix_util::assert_f64_eq;
    use std::collections::HashMap;
    use std::fs::File;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[test]
    fn real_density_function() {
        // let mut worldgen_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        // worldgen_path.push("../../generated/data/minecraft/worldgen");
        // 
        // let mut df_resolver = Resolver::new(
        //     deserializers(),
        //     Box::new(
        //         CascadeValueResolver::new(
        //             vec![
        //                 Box::new(MockValueResolver::new(HashMap::from([
        //                     (
        //                         "minecraft:overworld/depth".to_owned(),
        //                         read_json(worldgen_path.join("density_function/overworld/depth.json"))
        //                     ),
        //                     (
        //                         "minecraft:overworld/offset".to_owned(),
        //                         read_json(worldgen_path.join("density_function/overworld/offset.json"))
        //                     ),
        //                     (
        //                         "minecraft:overworld/continents".to_owned(),
        //                         read_json(worldgen_path.join("density_function/overworld/continents.json"))
        //                     ),
        //                     (
        //                         "minecraft:continentalness".to_owned(),
        //                         read_json(worldgen_path.join("noise/continentalness.json"))
        //                     ),
        //                     (
        //                         "minecraft:shift_x".to_owned(),
        //                         read_json(worldgen_path.join("density_function/shift_x.json"))
        //                     ),
        //                     (
        //                         "minecraft:offset".to_owned(),
        //                         read_json(worldgen_path.join("noise/offset.json"))
        //                     ),
        //                     (
        //                         "minecraft:shift_z".to_owned(),
        //                         read_json(worldgen_path.join("density_function/shift_z.json"))
        //                     ),
        //                     (
        //                         "minecraft:overworld/erosion".to_owned(),
        //                         read_json(worldgen_path.join("density_function/overworld/erosion.json"))
        //                     ),
        //                     (
        //                         "minecraft:erosion".to_owned(),
        //                         read_json(worldgen_path.join("noise/erosion.json"))
        //                     ),
        //                     (
        //                         "minecraft:overworld/ridges_folded".to_owned(),
        //                         read_json(worldgen_path.join("density_function/overworld/ridges_folded.json"))
        //                     ),
        //                     (
        //                         "minecraft:overworld/ridges".to_owned(),
        //                         read_json(worldgen_path.join("density_function/overworld/ridges.json"))
        //                     ),
        //                     (
        //                         "minecraft:ridge".to_owned(),
        //                         read_json(worldgen_path.join("noise/ridge.json"))
        //                     ),
        //                     (
        //                         "minecraft:overworld/factor".to_owned(),
        //                         read_json(worldgen_path.join("density_function/overworld/factor.json"))
        //                     ),
        //                 ]))),
        //             ]
        //         )
        //     ),
        // );
        // 
        // let file = File::open(worldgen_path.join("noise_settings/overworld.json")).unwrap();
        // let json: Value = serde_json::from_reader(file).unwrap();
        // 
        // let root_obj = json.as_object().unwrap();
        // let noise_router = root_obj.get("noise_router").unwrap();
        // let final_density = noise_router.as_object().unwrap();
        // let final_density = final_density.get("initial_density_without_jaggedness").unwrap();
        // 
        // let mut rng = XoroShiro::new(1);
        // 
        // let df = df_resolver.resolve(&final_density).unwrap();
        // let df = df.map(&SetupNoiseMapper::new(Arc::new(rng.fork_pos())));
        // 
        // let mut ctx = &mut DensityFunctionContext::default();
        // ctx.interpolation_counter = 1;
        // let sampled = df.sample(Vector3::new(2, 190, 10), &mut ctx);
        // 
        // assert_f64_eq!(-5.304568835016369, sampled, 7);
    }

    fn read_json(path: PathBuf) -> Value {
        serde_json::from_reader(File::open(path).unwrap()).unwrap()
    }
}
