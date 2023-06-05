use cgmath::*;
use instant::Duration;

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
}

impl LightController {
    pub fn new(angular_velocity: f32) -> Self {
        Self { angular_velocity }
    }
    pub fn update_light(&self, light: &mut Light, dt: Duration) {
        let old_position: cgmath::Vector3<_> = light.position.to_vec();
        light.position = Point3::from_vec(
            cgmath::Quaternion::from_axis_angle(
                (0.0, 1.0, 0.0).into(),
                cgmath::Deg(self.angular_velocity * dt.as_secs_f32()),
            ) * old_position,
        )
    }
}
