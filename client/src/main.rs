use crate::engine::my::MyRenderer;
use crate::engine::{startup, App, AssetManager, Input, ShaderCompiler};
use crate::modes::{Loading, Mode};
use log::info;

pub mod engine;
pub mod modes;

fn main() {
    env_logger::init();
    info!("OS: {}", std::env::consts::OS);

    let compiler = ShaderCompiler::new();
    compiler.compile_file("./assets/shaders/triangle.frag");
    compiler.compile_file("./assets/shaders/triangle.vert");
    startup::<Appplication>("Farmisto".to_string());
    info!("Bye!");
}

struct Appplication {
    mode: Box<dyn Mode>,
}

impl App for Appplication {
    fn start(assets: &mut AssetManager) -> Self {
        let editor = option_env!("FARMISTO_EDITOR").is_some();
        info!("Editor mode: {}", editor);
        let mut mode = Loading::new(editor);
        info!("Start {:?}", mode.name());
        mode.start(assets);

        Self { mode }
    }

    fn update(&mut self, input: Input, renderer: &mut MyRenderer, assets: &mut AssetManager) {
        self.mode.update(&input, renderer);
        if let Some(next) = self.mode.transition() {
            info!("Finish {:?}", self.mode.name());
            self.mode.finish();
            self.mode = next;
            info!("Start {:?}", self.mode.name());
            self.mode.start(assets);
        }
    }
}
