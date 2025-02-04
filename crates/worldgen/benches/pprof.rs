use criterion::profiler::Profiler;
// use pprof::ProfilerGuard;
use std::fs::File;
use std::os::raw::c_int;
use std::path::Path;

pub struct FlamegraphProfiler {
    frequency: c_int,
    // active_profiler: Option<ProfilerGuard<'a>>,
}

impl FlamegraphProfiler {
    #[allow(dead_code)]
    pub fn new(frequency: c_int) -> Self {
        FlamegraphProfiler {
            frequency,
            // active_profiler: None,
        }
    }
}

impl Profiler for FlamegraphProfiler {
    fn start_profiling(&mut self, _benchmark_id: &str, _benchmark_dir: &Path) {
        // self.active_profiler = Some(ProfilerGuard::new(self.frequency).unwrap());
    }

    fn stop_profiling(&mut self, _benchmark_id: &str, benchmark_dir: &Path) {
        std::fs::create_dir_all(benchmark_dir).unwrap();

        let flamegraph_path = benchmark_dir.join("flamegraph.svg");
        let flamegraph_file = File::create(&flamegraph_path)
            .expect("File system error while creating flamegraph.svg");
        // if let Some(profiler) = self.active_profiler.take() {
        //     let report = profiler
        //         .report()
        //         .build()
        //         .unwrap();
        // 
        //     report
        //         .flamegraph(flamegraph_file)
        //         .expect("Error writing flamegraph");
        // }
    }
}
