use std::marker::PhantomData;

use cgmath::*;
use instant::Duration;
use wgpu::util::DeviceExt;

pub struct Light {
    pub position: Point3<f32>,
    pub color: [f32; 3],
    pub controller: Controller,
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    hidden: PhantomData<()>,
}

pub struct LightBuilder<P, C, L> {
    position: P,
    color: C,
    control: L,
}

impl<P, C, L> LightBuilder<P, C, L> {
    pub fn position<V: Into<Point3<f32>>>(self, position: V) -> LightBuilder<Point3<f32>, C, L> {
        LightBuilder {
            position: position.into(),
            color: self.color,
            control: self.control,
        }
    }

    pub fn color(self, color: [f32; 3]) -> LightBuilder<P, [f32; 3], L> {
        LightBuilder {
            position: self.position,
            color,
            control: self.control,
        }
    }

    pub fn controller(self, angular_velocity: f32) -> LightBuilder<P, C, Controller> {
        LightBuilder {
            position: self.position,
            color: self.color,
            control: Controller::new(angular_velocity),
        }
    }
}

impl LightBuilder<Point3<f32>, [f32; 3], Controller> {
    pub fn finalize(self, device: &wgpu::Device) -> Light {
        let uniform = FlatLight {
            position: self.position.into(),
            _padding: 0,
            color: self.color,
            _padding2: 0,
        };

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light VB"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            label: None,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: None,
        });

        Light {
            position: self.position,
            color: self.color,
            controller: self.control,
            bind_group_layout,
            bind_group,
            buffer,
            hidden: PhantomData,
        }
    }
}

impl Light {
    pub fn build() -> LightBuilder<Option<Point3<f32>>, Option<[f32; 3]>, Option<Controller>> {
        LightBuilder {
            position: None,
            color: None,
            control: None,
        }
    }

    pub fn update(&mut self, dt: Duration, queue: &wgpu::Queue) {
        let old_position: cgmath::Vector3<_> = self.position.to_vec();

        self.position = Point3::from_vec(
            cgmath::Quaternion::from_axis_angle(
                (0.0, 1.0, 0.0).into(),
                cgmath::Deg(self.controller.angular_velocity * dt.as_secs_f32()),
            ) * old_position,
        );

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.flattened()]));
    }

    fn flattened(&self) -> FlatLight {
        FlatLight {
            position: self.position.into(),
            _padding: 0,
            color: self.color,
            _padding2: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct FlatLight {
    pub position: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    pub _padding: u32, // could use [f32;4] instead
    pub color: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    pub _padding2: u32,
}

pub struct Controller {
    angular_velocity: f32,
}

impl Controller {
    pub fn new(angular_velocity: f32) -> Self {
        Self { angular_velocity }
    }
}
