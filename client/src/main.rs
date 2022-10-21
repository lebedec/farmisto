use crate::engine::scene::SceneRenderer;
use crate::engine::{startup, App, Assets, Input, ShaderCompiler};
use crate::intro::Intro;
use crate::mode::Mode;
use glam::{EulerRot, Mat4, Quat, Vec3, Vec4};
use libfmod::Studio;
use log::info;
use std::io::Write;
use std::time::Instant;

pub mod animatoro;
pub mod bumaga;
pub mod editor;
pub mod engine;
pub mod gameplay;
pub mod intro;
pub mod menu;
pub mod mode;

fn main() {
    let start = Instant::now();
    env_logger::builder()
        .format(move |buf, record| {
            writeln!(
                buf,
                "{:.4} {}: {}",
                Instant::now().duration_since(start).as_secs_f32(),
                record.level(),
                record.args()
            )
        })
        .init();
    info!("OS: {}", std::env::consts::OS);
    startup::<Application>("Farmisto".to_string());
    info!("Bye!");
}

struct Application {
    mode: Box<dyn Mode>,
    time: f32,
}

impl App for Application {
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
        renderer: &mut SceneRenderer,
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
