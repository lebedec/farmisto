use game::Game;
use log::info;
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    env_logger::init();
    let mut game = Game::new();
    info!("OS: {}", std::env::consts::OS);
    let mut t = Instant::now();
    loop {
        let time = t.elapsed();
        t = Instant::now();
        game.update(time.as_secs_f32());
        thread::sleep(Duration::from_secs(5))
    }
    // info!("Bye!");
}
