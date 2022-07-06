use crate::engine::uniform::CameraUniform;
use crate::Input;
use glam::{vec3, Mat4, Vec3};
use sdl2::keyboard::Keycode;

pub struct Camera {
    viewport: [f32; 2],
    eye: Vec3,
}

impl Camera {
    pub fn new(viewport: [f32; 2]) -> Self {
        Self {
            viewport,
            eye: vec3(0.0, -2.0, -3.0),
        }
    }

    pub fn update(&mut self, input: &Input) {
        let mut offset = vec3(0.0, 0.0, 0.0);

        if input.down(Keycode::A) {
            offset.x -= 1.0;
        }
        if input.down(Keycode::D) {
            offset.x += 1.0;
        }
        if input.down(Keycode::W) {
            offset.y -= 1.0;
        }
        if input.down(Keycode::S) {
            offset.y += 1.0;
        }
        if input.down(Keycode::R) {
            offset.z += 1.0;
        }
        if input.down(Keycode::F) {
            offset.z -= 1.0;
        }

        self.eye += offset.normalize_or_zero() * input.time * 5.0;
    }

    pub fn uniform(&self) -> CameraUniform {
        // GLM was originally designed for OpenGL,
        // where the Y coordinate of the clip coordinates is inverted
        let inverted = Mat4::from_scale(vec3(1.0, -1.0, 1.0));

        CameraUniform {
            model: Mat4::IDENTITY,
            view: Mat4::look_at_rh(
                self.eye, // Vulkan Z: inside screen
                vec3(0.0, 0.0, 0.0),
                vec3(0.0, -1.0, 0.0), // Vulkan Y: bottom screen
            ),
            proj: Mat4::perspective_rh(
                45.0_f32.to_radians(),
                self.viewport[0] / self.viewport[1] as f32,
                0.1,
                100.0,
            ) * inverted,
        }
    }
}
