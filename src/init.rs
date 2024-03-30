pub async fn init(
    window: &winit::window::Window,
) -> (
    wgpu::Surface,
    wgpu::Device,
    wgpu::Queue,
    wgpu::SurfaceConfiguration,
) {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        dx12_shader_compiler: Default::default(),
        flags: wgpu::InstanceFlags::default(),
        gles_minor_version: wgpu::Gles3MinorVersion::default(),
    });

    // let window_handle = window.window_handle().unwrap().as_raw();
    // let display_handle = window.display_handle().unwrap();

    let surface = unsafe { instance.create_surface(&window).unwrap() };

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                //limits: wgpu::Limits::downlevel_webgl2_defaults()
                limits: wgpu::Limits::default(),
            },
            None, // Trace path
        )
        .await
        .unwrap();

    let size = window.inner_size();

    let capabilities = surface.get_capabilities(&adapter);

    // let format = capabilities
    //     .formats
    //     .iter()
    //     .copied()
    //     .find(|&f| f == wgpu::TextureFormat::Bgra8UnormSrgb)
    //     .unwrap();
    let format = *capabilities.formats.first().unwrap();

    dbg!(&capabilities.formats);

    // let config = surface.get_default_config(&adapter, size.width, size.height).unwrap();
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: size.width,
        height: size.height,
        present_mode: capabilities.present_modes[0],
        alpha_mode: capabilities.alpha_modes[0],
        view_formats: vec![],
    };

    surface.configure(&device, &config);

    (surface, device, queue, config)
}
