extern crate core;

use crate::engine::scene::SceneRenderer;
use crate::engine::{startup, App, Assets, Frame, Input, ShaderCompiler};
use crate::intro::Intro;
use crate::mode::Mode;
use log::info;
use std::io::Write;
use std::time::Instant;

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
}

impl App for Application {
    fn start(assets: &mut Assets) -> Self {
        let mut mode = Intro::new();
        mode.start(assets);
        Self { mode }
    }

    fn update(&mut self, context: Frame) {
        if let Some(next) = self.mode.transition() {
            info!("Finish {:?}", self.mode.name());
            self.mode.finish();
            self.mode = next;
            info!("Start {:?}", self.mode.name());
            self.mode.start(context.assets);
        }
        self.mode.update(context);
    }
}
