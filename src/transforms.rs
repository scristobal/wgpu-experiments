use cgmath::prelude::*;
use instant::Duration;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FlatTransform {
    model_transform: [[f32; 4]; 4],
    normal_transform: [[f32; 3]; 3],
}

pub struct Transform {
    translation: cgmath::Vector3<f32>,
    rotation: cgmath::Quaternion<f32>,
    scale: f32,
}

impl Transform {
    fn fattened(&self) -> FlatTransform {
        let model = cgmath::Matrix4::from_translation(self.translation)
            * cgmath::Matrix4::from(self.rotation)
            * cgmath::Matrix4::from_scale(self.scale);

        FlatTransform {
            model_transform: model.into(),
            normal_transform: cgmath::Matrix3::from(self.rotation).into(),
        }
    }
}

pub fn generate_transforms(rows: u32, cols: u32) -> Vec<Transform> {
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

pub struct Transforms {
    pub transforms: Vec<Transform>,
    pub buffer: wgpu::Buffer,
    pub number: u32,
}

pub struct TransformsBuilder<T> {
    transforms: T,
}

impl<T> TransformsBuilder<T> {
    pub fn transform_field(self, n: u32, m: u32) -> TransformsBuilder<Vec<Transform>> {
        let transforms = generate_transforms(n, m);

        TransformsBuilder { transforms }
    }
}

impl TransformsBuilder<Vec<Transform>> {
    pub fn finalize(self, device: &wgpu::Device) -> Transforms {
        let transforms = self
            .transforms
            .iter()
            .map(Transform::fattened)
            .collect::<Vec<_>>();

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("instance_buffer"),
            contents: bytemuck::cast_slice(&transforms),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let number = self.transforms.len() as u32;

        Transforms {
            transforms: self.transforms,
            buffer,
            number,
        }
    }
}

impl Transforms {
    pub fn build() -> TransformsBuilder<Option<Vec<Transform>>> {
        TransformsBuilder { transforms: None }
    }

    pub fn update(&mut self, _dt: Duration, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.flattened()));
    }

    fn flattened(&self) -> Vec<FlatTransform> {
        self.transforms.iter().map(Transform::fattened).collect()
    }
}
