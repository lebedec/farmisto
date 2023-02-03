use crate::engine::buffers::CameraUniform;
use crate::Input;
use glam::{vec3, Mat4, Vec3};
use sdl2::keyboard::Keycode;

pub struct Camera {
    pub eye: Vec3,
    z_near: f32,
    z_far: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            eye: vec3(0.0, -4.0, -4.0),
            z_near: 0.1,
            z_far: 10000.0,
        }
    }

    // pub fn cast_ray(&self, mouse: Cursor) -> (Vec3, Option<Vec3>) {
    //     let uniform = self.uniform();
    //     let inverted = (uniform.proj * uniform.view).inverse();
    //
    //     // todo: dramatic accuracy error on low z_far values
    //     let point =
    //         inverted.transform_point3(Vec3::new(mouse.viewport[0], -mouse.viewport[1], 1.0));
    //     let ray_dir = point.normalize_or_zero();
    //     let ray_origin = self.eye;
    //
    //     let normal = Vec3::new(0.0, 1.0, 0.0);
    //     let denom = normal.dot(ray_dir);
    //     let mut hit = None;
    //     if denom.abs() > 0.0001 {
    //         let t = (-ray_origin).dot(normal) / denom;
    //         if t >= 0.0 {
    //             hit = Some(ray_origin + ray_dir * t);
    //         }
    //     }
    //
    //     // info!("Mouse {:?} HIT {:?}", mouse.viewport, hit);
    //     (ray_dir, hit)
    // }

    pub fn update(&mut self, input: &Input) {
        let mut offset = vec3(0.0, 0.0, 0.0);

        if input.down(Keycode::A) {
            offset.x -= 1.0;
        }
        if input.down(Keycode::D) {
            offset.x += 1.0;
        }
        if input.down(Keycode::W) {
            offset.y += 1.0;
        }
        if input.down(Keycode::S) {
            offset.y -= 1.0;
        }
        if input.down(Keycode::R) {
            offset.z += 1.0;
        }
        if input.down(Keycode::F) {
            offset.z -= 1.0;
        }

        self.eye += offset.normalize_or_zero() * input.time * 1024.0;
    }

    pub fn position(&self) -> [f32; 2] {
        [self.eye.x, self.eye.y]
    }

    pub fn uniform(&self, width: f32, height: f32) -> CameraUniform {
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
                width / height,
                self.z_near,
                self.z_far,
            ) * inverted,
        }
    }
}
