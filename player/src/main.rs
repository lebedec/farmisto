use crate::input::Input;
use crate::modes::Loading;
use log::info;
use std::collections::HashMap;
use std::ffi::CString;
use std::thread;
use std::time::{Duration, Instant};

pub mod input;
pub mod modes;

trait Mode {
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    fn start(&mut self) {}

    #[allow(unused_variables)]
    fn update(&mut self, input: &Input) {}

    fn transition(&self) -> Option<Box<dyn Mode>> {
        None
    }

    fn finish(&mut self) {}
}

fn main() {
    env_logger::init();
    info!("OS: {}", std::env::consts::OS);

    let editor = option_env!("FARMISTO_EDITOR").is_some();
    info!("Editor mode: {}", editor);

    #[cfg(windows)]
    unsafe {
        winapi::um::shellscalingapi::SetProcessDpiAwareness(1);
    }
    let system = sdl2::init().unwrap();
    let video = system.video().unwrap();
    let window_size = [960, 540];
    let window = video
        .window(
            &format!("Farmisto [editor:{}] {}", editor, "0.0.1"),
            window_size[0],
            window_size[1],
        )
        .allow_highdpi()
        .position(0, 0)
        .vulkan()
        .build()
        .unwrap();
    info!(
        "SDL display: {:?}, dpi {:?}",
        video.display_bounds(0).unwrap(),
        video.display_dpi(0).unwrap(),
    );
    info!("Vulkan drawable: {:?}", window.vulkan_drawable_size());
    let mut event_pump = system.event_pump().unwrap();
    let instance_extensions: Vec<CString> = window
        .vulkan_instance_extensions()
        .unwrap()
        .iter()
        .map(|&extension| CString::new(extension).unwrap())
        .collect();
    info!("Vulkan extensions: {:?}", instance_extensions);

    let mut mode: Box<dyn Mode> = Loading::new(editor);
    info!("Start {:?}", mode.name());
    mode.start();

    let mut time = Instant::now();
    let mut input = Input::new();
    loop {
        input.reset();
        input.time = time.elapsed().as_secs_f32();
        time = Instant::now();
        for event in event_pump.poll_iter() {
            input.handle(event);
        }

        if input.terminating {
            break;
        }

        mode.update(&input);
        if let Some(next) = mode.transition() {
            info!("Finish {:?}", mode.name());
            mode.finish();
            mode = next;
            info!("Start {:?}", mode.name());
            mode.start();
        }

        thread::sleep(Duration::from_millis(16))
    }
    info!("Bye!");
}
