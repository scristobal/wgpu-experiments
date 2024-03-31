use cgmath::*;
use instant::Duration;
use std::f32::consts::FRAC_PI_2;
use wgpu::util::DeviceExt;

use crate::controller::Controller;

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

    pub fn update(&mut self, controller: &mut Controller, dt: Duration) {
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

pub struct ViewBuilder<C, P, L> {
    camera: C,
    projection: P,
    controller: L,
}

impl<C, P, L> ViewBuilder<C, P, L> {
    pub fn camera<V: Into<Point3<f32>>, Y: Into<Rad<f32>>, Pt: Into<Rad<f32>>>(
        self,
        position: V,
        yaw: Y,
        pitch: Pt,
    ) -> ViewBuilder<Camera, P, L> {
        ViewBuilder {
            camera: Camera::new(position, yaw, pitch),
            projection: self.projection,
            controller: self.controller,
        }
    }

    pub fn projection<F: Into<Rad<f32>>>(
        self,
        width: u32,
        height: u32,
        fovy: F,
        znear: f32,
        zfar: f32,
    ) -> ViewBuilder<C, Projection, L> {
        ViewBuilder {
            projection: Projection::new(width, height, fovy, znear, zfar),
            camera: self.camera,
            controller: self.controller,
        }
    }

    pub fn controller(self, controller: Controller) -> ViewBuilder<C, P, Controller> {
        ViewBuilder {
            controller,
            camera: self.camera,
            projection: self.projection,
        }
    }
}

impl ViewBuilder<Camera, Projection, Controller> {
    pub fn finalize(self, device: &wgpu::Device) -> View {
        let uniform = ViewUniform::new(&self.camera, &self.projection);

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("View Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("View bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("View bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        View {
            camera: self.camera,
            projection: self.projection,
            controller: self.controller,
            buffer,
            bind_group_layout,
            bind_group,
        }
    }
}

pub struct View {
    pub camera: Camera,
    pub projection: Projection,
    pub controller: Controller,
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl View {
    pub fn build() -> ViewBuilder<Option<Camera>, Option<Projection>, Option<Controller>> {
        ViewBuilder {
            camera: None,
            projection: None,
            controller: None,
        }
    }

    pub fn update(&mut self, dt: Duration, queue: &wgpu::Queue) {
        self.camera.update(&mut self.controller, dt);

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.as_uniform()]));
    }

    fn as_uniform(&self) -> ViewUniform {
        ViewUniform::new(&self.camera, &self.projection)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ViewUniform {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

impl ViewUniform {
    fn new(camera: &Camera, projection: &Projection) -> Self {
        Self {
            view_position: camera.position.to_homogeneous().into(),
            view_proj: (projection.calc_matrix() * camera.calc_matrix()).into(),
        }
    }
}
