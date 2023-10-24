use std::f32::consts::FRAC_PI_3;

use ash::vk;
use nalgebra::{Matrix4, Rotation3, Unit, Vector3};

use crate::buffer::Buffer;

use super::camera_builder::CameraBuilder;

pub struct Camera {
    pub view_matrix: Matrix4<f32>,
    pub position: Vector3<f32>,
    pub view_direction: Unit<Vector3<f32>>,
    pub down_direction: Unit<Vector3<f32>>,
    pub fovy: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
    pub projection_matrix: Matrix4<f32>,
}

impl Camera {
    pub fn builder() -> CameraBuilder {
        CameraBuilder {
            position: Vector3::new(0.0, -3.0, -3.0),
            view_direction: Unit::new_normalize(Vector3::new(0.0, 1.0, 1.0)),
            down_direction: Unit::new_normalize(Vector3::new(0.0, 1.0, -1.0)),
            fovy: FRAC_PI_3,
            aspect: 800. / 600.,
            near: 0.1,
            far: 100.,
        }
    }
    pub fn update_buffer(
        &self,
        logical_device: &ash::Device,
        memory_properties: vk::PhysicalDeviceMemoryProperties,
        buffer: &mut Buffer,
    ) {
        let data: [[[f32; 4]; 4]; 2] = [self.view_matrix.into(), self.projection_matrix.into()];
        buffer
            .fill(logical_device, &data, memory_properties)
            .unwrap();
    }

    pub fn move_forward(&mut self, distance: f32) {
        self.position += distance * self.view_direction.as_ref();
        self.update_view_matrix();
    }

    pub fn move_backward(&mut self, distance: f32) {
        self.move_forward(-distance);
    }

    pub fn turn_right(&mut self, angle: f32) {
        let rotation = Rotation3::from_axis_angle(&self.down_direction, angle);
        self.view_direction = rotation * self.view_direction;
        self.update_view_matrix();
    }

    pub fn turn_left(&mut self, angle: f32) {
        self.turn_right(-angle);
    }

    pub fn turn_up(&mut self, angle: f32) {
        let right = Unit::new_normalize(self.down_direction.cross(&self.view_direction));
        let rotation = Rotation3::from_axis_angle(&right, angle);
        self.view_direction = rotation * self.view_direction;
        self.down_direction = rotation * self.down_direction;
        self.update_view_matrix();
    }

    pub fn turn_down(&mut self, angle: f32) {
        self.turn_up(-angle);
    }

    pub fn update_view_matrix(&mut self) {
        let right = Unit::new_normalize(self.down_direction.cross(&self.view_direction));
        let m = Matrix4::new(
            right.x,
            right.y,
            right.z,
            -right.dot(&self.position), //
            self.down_direction.x,
            self.down_direction.y,
            self.down_direction.z,
            -self.down_direction.dot(&self.position), //
            self.view_direction.x,
            self.view_direction.y,
            self.view_direction.z,
            -self.view_direction.dot(&self.position), //
            0.0,
            0.0,
            0.0,
            1.0,
        );
        self.view_matrix = m;
    }

    pub fn update_projection_matrix(&mut self) {
        let d = 1.0 / (0.5 * self.fovy).tan();
        self.projection_matrix = Matrix4::new(
            d / self.aspect,
            0.0,
            0.0,
            0.0,
            0.0,
            d,
            0.0,
            0.0,
            0.0,
            0.0,
            self.far / (self.far - self.near),
            -self.near * self.far / (self.far - self.near),
            0.0,
            0.0,
            1.0,
            0.0,
        );
    }
}
