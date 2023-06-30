use cgmath::*;
use instant::Duration;
use wgpu::util::DeviceExt;

pub struct Light {
    pub position: Point3<f32>,
    pub color: [f32; 3],
}

impl Light {
    pub fn new<V: Into<Point3<f32>>>(position: V, color: [f32; 3]) -> Self {
        Self {
            position: position.into(),
            color,
        }
    }

    pub fn uniform(&self) -> LightUniform {
        LightUniform {
            position: self.position.into(),
            _padding: 0,
            color: self.color,
            _padding2: 0,
        }
    }
}

pub struct LightBuffer {
    pub buffer: wgpu::Buffer,
}

impl LightBuffer {
    pub fn new(device: &wgpu::Device, light: &Light) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light VB"),
            contents: bytemuck::cast_slice(&[light.uniform()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Self { buffer }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub position: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    pub _padding: u32, // could use [f32;4] instead
    pub color: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    pub _padding2: u32,
}

pub struct LightController {
    angular_velocity: f32,
    pub light: Light,
}

impl LightController {
    pub fn new(angular_velocity: f32, light: Light) -> Self {
        Self {
            angular_velocity,
            light,
        }
    }
    pub fn update(&mut self, dt: Duration) {
        let old_position: cgmath::Vector3<_> = self.light.position.to_vec();

        self.light.position = Point3::from_vec(
            cgmath::Quaternion::from_axis_angle(
                (0.0, 1.0, 0.0).into(),
                cgmath::Deg(self.angular_velocity * dt.as_secs_f32()),
            ) * old_position,
        );
    }
}

pub trait UpdateBuffer {
    fn update(&self, queue: &wgpu::Queue, buffer: &wgpu::Buffer);
}

impl UpdateBuffer for Light {
    fn update(&self, queue: &wgpu::Queue, buffer: &wgpu::Buffer) {
        let light_uniform = self.uniform();
        queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[light_uniform]));
    }
}
