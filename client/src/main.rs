use crate::engine::my::MyRenderer;
use crate::engine::{startup, App, Assets, Input, ShaderCompiler};
use crate::intro::Intro;
use crate::mode::Mode;
use libfmod::Studio;
use log::info;

pub mod editor;
pub mod engine;
pub mod gameplay;
pub mod intro;
pub mod menu;
pub mod mode;

fn main() {
    env_logger::init();
    info!("OS: {}", std::env::consts::OS);
    startup::<Appplication>("Farmisto".to_string());
    info!("Bye!");
}

struct Appplication {
    mode: Box<dyn Mode>,
    time: f32,
}

impl App for Appplication {
    fn start(assets: &mut Assets) -> Self {
        let editor = option_env!("FARMISTO_EDITOR").is_some();
        info!("Editor mode: {}", editor);
        let mut mode = Intro::new(editor);
        info!("Start {:?}", mode.name());
        mode.start(assets);

        Self { mode, time: 0.0 }
    }

    fn update(
        &mut self,
        input: Input,
        renderer: &mut MyRenderer,
        assets: &mut Assets,
        studio: &Studio,
    ) {
        self.time += input.time;
        if self.time > 1.0 {
            self.time = 0.0;
            // info!("fire event!");
            // let event = studio.get_event("event:/Farmer/Footsteps").unwrap();
            // // studio.set_listener_attributes()
            // let event = event.create_instance().unwrap();
            // event.set_parameter_by_name("Terrain", 0.0, false).unwrap();
            // event.start().unwrap();
            // event.release().unwrap();
        }
        self.mode.update(&input, renderer, assets);
        if let Some(next) = self.mode.transition(renderer) {
            info!("Finish {:?}", self.mode.name());
            self.mode.finish();
            self.mode = next;
            info!("Start {:?}", self.mode.name());
            self.mode.start(assets);
        }
    }
}
