extern crate core;

use crate::assets::Assets;
use crate::engine::{startup, App, Frame, Input};
use crate::intro::Intro;
use crate::mode::Mode;
use log::info;
use std::io::Write;
use std::time::Instant;

pub mod assets;
pub mod engine;
pub mod gameplay;
pub mod intro;
pub mod menu;
pub mod mode;
pub mod monitoring;

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
    fn start(_assets: &mut Assets) -> Self {
        let mut mode = Intro::new();
        Self { mode }
    }

    fn update(&mut self, frame: &mut Frame) {
        if let Some(next) = self.mode.transition(frame) {
            info!("Finish {:?}", self.mode.name());
            self.mode.finish();
            info!("Start {:?}", self.mode.name());
            self.mode = next;
        }
        self.mode.update(frame);
    }
}
