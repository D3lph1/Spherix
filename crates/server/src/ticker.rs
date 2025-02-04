use bevy_app::{App, AppExit, Plugin, PluginsState, ScheduleRunnerPlugin};
use bevy_ecs::event::ManualEventReader;
use std::slice::IterMut;
use std::sync::Arc;
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime};

use bevy_ecs::prelude::{Events, Schedule, World};
use tracing::{trace, warn};

use crate::server::Server;

const TICK_WARNING_WINDOW: Duration = Duration::from_secs(5);

pub struct TickerPlugin {
    tick_rate: usize,
}

impl Plugin for TickerPlugin {
    fn build(&self, app: &mut App) {
        let tick_duration = Duration::from_millis((1000 / self.tick_rate) as u64);

        app.set_runner(move |mut app: App| {
            let plugins_state = app.plugins_state();
            if plugins_state != PluginsState::Cleaned {
                while app.plugins_state() == PluginsState::Adding {
                    bevy_tasks::tick_global_task_pools_on_main_thread();
                }
                app.finish();
                app.cleanup();
            }

            let mut app_exit_event_reader = ManualEventReader::<AppExit>::default();
            let mut tick = move |app: &mut App| -> Result<Duration, AppExit> {
                let start_time = Instant::now();

                app.update();

                if let Some(app_exit_events) =
                    app.world.get_resource_mut::<Events<AppExit>>()
                {
                    if let Some(exit) = app_exit_event_reader.read(&app_exit_events).last()
                    {
                        return Err(exit.clone());
                    }
                }

                let end_time = Instant::now();

                return Ok(end_time - start_time);
            };

            let mut last_tick_warning_at: Instant = Instant::now() - TICK_WARNING_WINDOW * 2;
            let mut tick_counter = 0;

            {
                while let Ok(diff) = tick(&mut app) {
                    if diff < tick_duration {
                        sleep(tick_duration - diff);
                    } else if diff > tick_duration && last_tick_warning_at.elapsed() > TICK_WARNING_WINDOW {
                        warn!("Too small tick rate. The tick took {:?}.", diff);
                        last_tick_warning_at = Instant::now();
                    }

                    tick_counter += 1;
                }
            }
        });
    }
}

impl TickerPlugin {
    pub fn new(tick_rate: usize) -> Self {
        Self {
            tick_rate,
        }
    }

    // fn tick(&mut self, world: &mut World, schedules: IterMut<Schedule>) {
    //     for schedule in schedules {
    //         schedule.run(world);
    //     }
    //     world.clear_trackers();
    //
    //     self.tick += 1;
    // }
}
