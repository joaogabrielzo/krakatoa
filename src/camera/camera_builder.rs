use nalgebra::{Vector3, Unit, Matrix4};

use super::camera::Camera;

pub struct CameraBuilder {
    pub position: Vector3<f32>,
    pub view_direction: Unit<Vector3<f32>>,
    pub down_direction: Unit<Vector3<f32>>,
    pub fovy: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl CameraBuilder {
    pub fn build(self) -> Camera {
        if self.far < self.near {
            println!(
                "far plane (at {}) closer than near plane (at {}) — is that right?",
                self.far, self.near
            );
        }
        let mut cam = Camera {
            position: self.position,
            view_direction: self.view_direction,
            down_direction: Unit::new_normalize(
                self.down_direction.as_ref()
                    - self
                        .down_direction
                        .as_ref()
                        .dot(self.view_direction.as_ref())
                        * self.view_direction.as_ref(),
            ),
            fovy: self.fovy,
            aspect: self.aspect,
            near: self.near,
            far: self.far,
            view_matrix: Matrix4::identity(),
            projection_matrix: Matrix4::identity(),
        };
        cam.update_projection_matrix();
        cam.update_view_matrix();
        cam
    }
    pub fn position(mut self, pos: Vector3<f32>) -> CameraBuilder {
        self.position = pos;
        self
    }
    pub fn fovy(mut self, fovy: f32) -> CameraBuilder {
        self.fovy = fovy.max(0.01).min(std::f32::consts::PI - 0.01);
        self
    }
    pub fn aspect(mut self, aspect: f32) -> CameraBuilder {
        self.aspect = aspect;
        self
    }
    pub fn near(mut self, near: f32) -> CameraBuilder {
        if near <= 0.0 {
            println!("setting near plane to negative value: {} — you sure?", near);
        }
        self.near = near;
        self
    }
    pub fn far(mut self, far: f32) -> CameraBuilder {
        if far <= 0.0 {
            println!("setting far plane to negative value: {} — you sure?", far);
        }
        self.far = far;
        self
    }
    pub fn view_direction(mut self, direction: Vector3<f32>) -> CameraBuilder {
        self.view_direction = Unit::new_normalize(direction);
        self
    }
    pub fn down_direction(mut self, direction: Vector3<f32>) -> CameraBuilder {
        self.down_direction = Unit::new_normalize(direction);
        self
    }
}
