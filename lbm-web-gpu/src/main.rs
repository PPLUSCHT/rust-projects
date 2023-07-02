use driver::Driver;
use lbm::Simulation;
use winit::{event_loop::{EventLoop, ControlFlow}, dpi::LogicalSize, event::{Event, WindowEvent}, window::Window};

pub mod lbm;
pub mod driver;

async fn run(driver: Driver, event_loop: EventLoop<()>, window:Window) {
    
   let mut sim = Simulation::<X_DIM, Y_DIM>::new(&driver, OMEGA).await;

   let swapchain_capabilities = driver.surface.get_capabilities(&driver.adapter);
   let swapchain_format = swapchain_capabilities.formats[0];

   let mut config = wgpu::SurfaceConfiguration {
       usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
       format: swapchain_format,
       width: driver.size.width,
       height: driver.size.height,
       present_mode: wgpu::PresentMode::Fifo,
       alpha_mode: swapchain_capabilities.alpha_modes[0],
       view_formats: vec![],
   };

   driver.surface.configure(&driver.device, &config);

   event_loop.run(move |event, _, control_flow| {
       *control_flow = ControlFlow::Poll;
       match event {
           Event::WindowEvent {
               event: WindowEvent::Resized(size),
               ..
           } => {
               // Reconfigure the surface with the new size
               config.width = size.width;
               config.height = size.height;
               driver.surface.configure(&driver.device, &config);
               // On macos the window needs to be redrawn manually after resizing
               window.request_redraw();
           }

           Event::WindowEvent {
               event:WindowEvent::MouseInput{
                   ..
               },
               ..
           } => {
               sim.iterate(&driver, 1);
           }

           // Event::RedrawRequested(_) => {
           //     sim.render(&driver);
           // }
           Event::RedrawEventsCleared => {
            sim.iterate(&driver, 1);
           }
           Event::WindowEvent {
               event: WindowEvent::CloseRequested,
               ..
           } => *control_flow = ControlFlow::Exit,
           _ => {}
       }
   });
}


const X_DIM:u32 = 200;
const Y_DIM:u32 = 200;
const OMEGA:f32 = 1.0/(0.15 + 0.5);
fn main() {
   let event_loop = EventLoop::new();
   let window = winit::window::WindowBuilder::new()
                                       .with_inner_size(LogicalSize{width: X_DIM as f32 * 5.0, height: Y_DIM as f32* 5.0})
                                       .build(&event_loop).unwrap();
   #[cfg(not(target_arch = "wasm32"))]
   {
       env_logger::init();
       let driver = pollster::block_on(Driver::new(&window));
       pollster::block_on(run(driver, event_loop, window));
   }
}