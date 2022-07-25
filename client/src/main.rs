use crate::engine::scene::SceneRenderer;
use crate::engine::{startup, App, Assets, Input, ShaderCompiler};
use crate::intro::Intro;
use crate::mode::Mode;
use glam::{EulerRot, Mat4, Quat, Vec3, Vec4};
use libfmod::Studio;
use log::info;

pub mod animatoro;
pub mod bumaga;
pub mod editor;
pub mod engine;
pub mod gameplay;
pub mod intro;
pub mod menu;
pub mod mode;

fn main() {
    // let a = Vec4::new(-1., -1., 0., 1.);
    // let b = Vec4::new(0., -1., 0., 1.);
    // let c = Vec4::new(0., 0., 0., 1.);
    //
    // // centered
    // let a = Vec4::new(-1., -0.5, 0., 1.);
    // let b = Vec4::new(0., -0.5, 0., 1.);
    // let c = Vec4::new(0., 0.5, 0., 1.);
    //
    // let bone = Mat4::from_translation(Vec3::new(0., -0.5, 0.));
    //
    // println!("Ap: {}", bone * a);
    // println!("Bp: {}", bone * b);
    // println!("Cp: {}", bone * c);
    //
    // let z_neg_45 = Quat::from_euler(
    //     EulerRot::XYZ,
    //     0.0f32.to_radians(),
    //     0.0f32.to_radians(),
    //     45.0f32.to_radians(),
    // );
    // let pose = Mat4::from_rotation_translation(z_neg_45, Vec3::ZERO) * bone;
    //
    // println!("A: {}", pose * a);
    // println!("B: {}", pose * b);
    // println!("C: {}", pose * c);
    //
    // // answer
    // // A: [0.00000011920929, -1.4142135, 0, 1]
    // // B: [0.7071068, -0.7071067, 0, 1]
    // // C: [0, 0, 0, 1]
    //
    // return;

    // let y_neg_45 = Quat::from_euler(
    //     EulerRot::XYZ,
    //     0.0f32.to_radians(),
    //     -45.0f32.to_radians(),
    //     0.0f32.to_radians(),
    // );
    //
    // println!("Z-45: {:?}", z_neg_45);
    // println!("Y-45: {:?}", y_neg_45);
    //
    // return;
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
