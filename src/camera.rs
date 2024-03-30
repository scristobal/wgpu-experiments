use cgmath::*;
use instant::Duration;
use std::f32::consts::FRAC_PI_2;
use winit::dpi::PhysicalPosition;
use winit::event::ElementState;
use winit::event::*;
use winit::keyboard::KeyCode;

use crate::controller::CameraController;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

#[derive(Debug)]
pub struct Camera {
    pub position: Point3<f32>,
    yaw: Rad<f32>,   // horizontal plane
    pitch: Rad<f32>, // vertical plane
}

impl Camera {
    pub fn new<V: Into<Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(
        position: V,
        yaw: Y,
        pitch: P,
    ) -> Self {
        Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
        }
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        Matrix4::look_to_rh(
            self.position,
            Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            Vector3::unit_y(),
        )
    }

    pub fn update(&mut self, controller: &mut CameraController, dt: Duration) {
        let dt = dt.as_secs_f32();

        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = self.yaw.0.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();

        self.position += forward
            * (controller.amount_forward - controller.amount_backward)
            * controller.speed
            * dt;
        self.position +=
            right * (controller.amount_right - controller.amount_left) * controller.speed * dt;

        // Move in/out (aka. "zoom")
        // Note: this isn't an actual zoom. The camera's position
        // changes when zooming. I've added this to make it easier
        // to get closer to an object you want to focus on.
        let (pitch_sin, pitch_cos) = self.pitch.0.sin_cos();
        let scrollward =
            Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
        self.position +=
            scrollward * controller.scroll * controller.speed * controller.sensitivity * dt;

        // Move up/down. Since we don't use roll, we can just
        // modify the y coordinate directly.
        self.position.y += (controller.amount_up - controller.amount_down) * controller.speed * dt;

        // Rotate
        self.yaw += Rad(controller.rotate_horizontal) * controller.sensitivity * dt;
        self.pitch += Rad(-controller.rotate_vertical) * controller.sensitivity * dt;

        // Keep the camera's angle from going too high/low.
        if self.pitch < -Rad(SAFE_FRAC_PI_2) {
            self.pitch = -Rad(SAFE_FRAC_PI_2);
        } else if self.pitch > Rad(SAFE_FRAC_PI_2) {
            self.pitch = Rad(SAFE_FRAC_PI_2);
        }

        controller.reset()
    }
}

#[derive(Debug)]
pub struct Projection {
    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

pub struct ViewBuilder {
    camera: Option<Camera>,
    projection: Option<Projection>,
}

impl ViewBuilder {
    pub fn new() -> Self {
        Self {
            camera: None,
            projection: None,
        }
    }
    pub fn set_camera<V: Into<Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(
        self,
        position: V,
        yaw: Y,
        pitch: P,
    ) -> Self {
        Self {
            camera: Some(Camera::new(position, yaw, pitch)),
            ..self
        }
    }

    pub fn set_projection<F: Into<Rad<f32>>>(
        self,
        width: u32,
        height: u32,
        fovy: F,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Self {
            projection: Some(Projection::new(width, height, fovy, znear, zfar)),
            ..self
        }
    }

    pub fn build(self) -> View {
        View {
            camera: self.camera.unwrap(),
            projection: self.projection.unwrap(),
        }
    }
}

pub struct View {
    pub camera: Camera,
    pub projection: Projection,
}

impl View {
    pub fn as_uniform(&self) -> CameraUniform {
        CameraUniform::new(&self.camera, &self.projection)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new(camera: &Camera, projection: &Projection) -> Self {
        Self {
            view_position: camera.position.to_homogeneous().into(),
            view_proj: (projection.calc_matrix() * camera.calc_matrix()).into(),
        }
    }
}
