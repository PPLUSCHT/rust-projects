use driver::Driver;
use lbm::Simulation;
use timeit::timeit_loops;
use winit::{event_loop::{EventLoop, ControlFlow}, dpi::LogicalSize, event::{Event, WindowEvent}, window::Window};

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

pub mod lbm;
pub mod driver;
pub mod barrier_shapes;

const X_DIM:u32 = 4096;
const Y_DIM:u32 = 1024;
const OMEGA:f32 = 1.0/(0.6 + 0.5);

pub async fn run_native(driver: Driver, event_loop: EventLoop<()>, window:Window) {

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
                let prev_frame_num = sim.frame_num as f64;
                let sec = timeit_loops!(1000, {
                    sim.iterate(&driver, 10);
                });
                println!("FPS over past 1000 loops: {}", (sim.frame_num as f64 - prev_frame_num)/(1000.0 * sec));
           }

           Event::WindowEvent {
               event: WindowEvent::CloseRequested,
               ..
           } => *control_flow = ControlFlow::Exit,
           _ => {}
       }
   });
}

pub async fn run_wasm(event_loop: EventLoop<()>, window:Window) {
    use web_sys::console;
    console::log_1(&"Hello from after run".into());
    let driver = Driver::new(&window).await;

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
    console::log_1(&"Prior to loop".into());
    event_loop.run(move |event, _, control_flow| {
        control_flow.set_wait();
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
                let prev_frame_num = sim.frame_num as f64;
                sim.iterate(&driver, 10);
            }
 
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
 }

#[wasm_bindgen]
pub fn runner() {
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
                        .with_inner_size(LogicalSize{width: X_DIM as f32 * 0.5, height: Y_DIM as f32 * 0.5})
                        .build(&event_loop).unwrap();
    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;
        use web_sys::console;
        use wasm_bindgen_futures;
        use web_sys;
        console_log::init().expect("could not initialize logger");
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");
        console::log_1(&"Hello from before run".into());
        wasm_bindgen_futures::spawn_local(run_wasm(event_loop, window));
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
        let driver = pollster::block_on(Driver::new(&window));
        pollster::block_on(run_native(driver, event_loop, window));
    }
}

#[wasm_bindgen]
#[cfg(target_arch = "wasm32")]
pub fn debug(){
    use web_sys::console;
    console::log_1(&"Hello using web-sys".into());
}