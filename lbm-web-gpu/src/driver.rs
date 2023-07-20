use wgpu::{Device, Features};

pub struct Driver{
    pub size: winit::dpi::PhysicalSize<u32>,
    pub surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl Driver{
    pub async fn new(window: &winit::window::Window) -> Driver{
    
    let size = window.inner_size();

    let instance = wgpu::Instance::default();

    let surface = unsafe { instance.create_surface(&window) }.unwrap();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            // Request an adapter which can render to our surface
            compatible_surface: Some(&surface),
        })
        .await
        .expect("Failed to find an appropriate adapter");

    #[cfg(target_arch = "wasm32")]
    {
        use web_sys::console;
        console::log_1(&format!("{:?}", adapter.limits()).into());
        console::log_1(&format!("{:?}", adapter.features()).into());   
    }

    // Create the logical device and command queue
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: adapter.features(),
                // features: wgpu::2Features::empty(),
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                limits: wgpu::Limits::default(),
            },
            None,
        )
        .await
        .expect("Failed to create device");

    return Driver{ size, surface, adapter, device, queue};
    }
}