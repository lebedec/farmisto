extern crate core;

use std::io::Write;
use std::time::Instant;

use log::info;

use crate::assets::Assets;
use crate::engine::{startup, App, Frame, Input};
use crate::intro::Intro;
use crate::mode::Mode;

pub mod assets;
pub mod bumaga;
pub mod engine;
pub mod gameplay;
pub mod intro;
pub mod menu;
pub mod mode;
pub mod monitoring;
pub mod translation;

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
        let mode = Intro::new();
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
