use bevy_ecs::prelude::Resource;

#[derive(Resource)]
pub struct Game {
    // pub schedule: Schedule
}

impl Game {
    pub fn new() -> Self {
        Self {
            // schedule
        }
    }

    // fn create_app() -> App {
    //     let mut app = App::new();
    //     app.add_system(on_join);
    //
    //     app
    // }
}
