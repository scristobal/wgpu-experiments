use cgmath::prelude::*;
use wgpu::util::DeviceExt;

pub trait Instance {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RawInstance {
    model: [[f32; 4]; 4],
    normal: [[f32; 3]; 3],
}

impl Instance for RawInstance {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<RawInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5, // 0 is position, 1 is tex_coords, 2,3 and 4 are reserved for later, hence start at 5
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub struct Transform {
    translation: cgmath::Vector3<f32>,
    rotation: cgmath::Quaternion<f32>,
    scale: f32,
}

impl Transform {
    fn to_raw(&self) -> RawInstance {
        let model = cgmath::Matrix4::from_translation(self.translation)
            * cgmath::Matrix4::from(self.rotation)
            * cgmath::Matrix4::from_scale(self.scale);

        RawInstance {
            model: model.into(),
            normal: cgmath::Matrix3::from(self.rotation).into(),
        }
    }
}

pub struct Instances {
    pub instance_buffer: wgpu::Buffer,
    pub num_instances: u32,
}

impl Instances {
    pub fn from_transforms(transforms: &[Transform], device: &wgpu::Device) -> Self {
        let instance_data = transforms.iter().map(Transform::to_raw).collect::<Vec<_>>();

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("instance_buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            instance_buffer,
            num_instances: transforms.len() as u32,
        }
    }
}

pub fn sample_transform_field(rows: u32, cols: u32) -> Vec<Transform> {
    const SPACE_BETWEEN: f32 = 4.0;
    (0..rows)
        .flat_map(|z| {
            (0..cols).map(move |x| {
                let x = SPACE_BETWEEN * (x as f32 - cols as f32 / 2.0);
                let z = SPACE_BETWEEN * (z as f32 - rows as f32 / 2.0);

                let translation = cgmath::Vector3 { x, y: 0.0, z };

                let rotation = if translation.is_zero() {
                    cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
                } else {
                    cgmath::Quaternion::from_axis_angle(translation.normalize(), cgmath::Deg(45.0))
                };

                let scale = 1.0;

                Transform {
                    translation,
                    rotation,
                    scale,
                }
            })
        })
        .collect::<Vec<_>>()
}
