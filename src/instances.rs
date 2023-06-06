use cgmath::prelude::*;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RawInstance {
    model_transform: [[f32; 4]; 4],
    normal_transform: [[f32; 3]; 3],
}

pub struct Transform {
    translation: cgmath::Vector3<f32>,
    rotation: cgmath::Quaternion<f32>,
    scale: f32,
}

impl Transform {
    pub fn to_raw(&self) -> RawInstance {
        let model = cgmath::Matrix4::from_translation(self.translation)
            * cgmath::Matrix4::from(self.rotation)
            * cgmath::Matrix4::from_scale(self.scale);

        RawInstance {
            model_transform: model.into(),
            normal_transform: cgmath::Matrix3::from(self.rotation).into(),
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
