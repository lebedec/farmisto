use game::Game;
use log::info;
use std::thread;
use std::time::Duration;

fn main() {
    env_logger::init();
    let mut game = Game::new();
    info!("OS: {}", std::env::consts::OS);
    loop {
        game.update();
        thread::sleep(Duration::from_secs(5))
    }
    // info!("Bye!");
}
