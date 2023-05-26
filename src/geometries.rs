use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        /*
        Alternative implementation w/ macros:
            use std::mem;
            const ATTRIBUTES: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];
            wgpu::VertexBufferLayout {
                array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &Self::ATTRIBUTES,
            }
        */
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

/**
note orders in `VERTICES` are counter-clock-wise because we used
`wgpu::FrontFace:CCw` in the `render_pipeline`
*/
const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.8, 0.8, 0.0],
        tex_coords: [0.0, 0.0],
    }, // 0
    Vertex {
        position: [0.8, 0.8, 0.0],
        tex_coords: [1.0, 0.0],
    }, // 1
    Vertex {
        position: [0.8, -0.8, 0.0],
        tex_coords: [1.0, 1.0],
    }, // 2
    Vertex {
        position: [-0.8, -0.8, 0.0],
        tex_coords: [0.0, 1.0],
    }, // 3
];

const INDICES: &[u16] = &[0, 3, 1, 1, 3, 2];

pub struct Vertices {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
}

impl Vertices {
    pub fn new(device: &wgpu::Device) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_indices = INDICES.len() as u32;

        Self {
            vertex_buffer,
            index_buffer,
            num_indices,
        }
    }
}
