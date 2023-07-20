use barrier_shapes::{Shape, blob::Blob, merge_shapes::merge, line};
use driver::Driver;
use lbm::ColorMap;
use web_sys::console;
use winit::{event_loop::{EventLoop, ControlFlow}, dpi::PhysicalSize, event::{Event, WindowEvent, ElementState}, window::Window};
use wasm_bindgen::prelude::*;

use lazy_static::lazy_static; // 1.4.0
use std::{sync::Mutex, collections::HashSet};

lazy_static! {
    static ref CURRENT_OUTPUT: Mutex<SummaryStat> = Mutex::new(SummaryStat::Curl);
    static ref BARRIER_CHANGE: Mutex<bool> = Mutex::new(false);
    static ref PAUSE: Mutex<bool> = Mutex::new(true);
    static ref OUTPUT_CHANGED: Mutex<bool> = Mutex::new(true);
    static ref COMPUTE_PER_RENDER: Mutex<u32> = Mutex::new(50);
    static ref VISCOSITY: Mutex<f32> = Mutex::new(0.1);
    static ref VISCOSITY_CHANGED: Mutex<bool> = Mutex::new(false);
    static ref CURRENT_COLOR_MAP: Mutex<ColorMap> = Mutex::new(ColorMap::Jet);
    static ref COLOR_CHANGED: Mutex<bool> = Mutex::new(false);
    static ref EQUILIBRIUM_RESET: Mutex<bool> = Mutex::new(false);
}

use crate::{lbm::SummaryStat, barrier_shapes::line::Line};

pub mod driver;
pub mod barrier_shapes;
pub mod lbm;

const X_DIM:u32 = 2560;
const Y_DIM:u32 = 1440;
const OMEGA:f32 = 1.0/(0.5 + 0.3);


pub async fn run_wasm(event_loop: EventLoop<()>, window:Window) {
    let driver = Driver::new(&window).await;

    let mut lbm = lbm::LBM::<X_DIM,Y_DIM>::new(&driver, OMEGA);

    let mut pressed = false; 
    let mut click_handler = ClickHandler::new();
    let mut current_position: (isize, isize) = (0,0);
 
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
        control_flow.set_poll();
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                config.width = size.width;
                config.height = size.height;
                driver.surface.configure(&driver.device, &config);
                window.request_redraw();
            }

            Event::WindowEvent { 
                event: WindowEvent::CursorMoved{position, ..}, ..  
            } => {
                console::log_1(&format!("Pressed from cursor move: {}", pressed).into());
                if pressed{
                    let temp:(i32, i32) = position.into();
                    current_position = (temp.0 as isize, temp.1 as isize);
                    click_handler.handle_movement((temp.0 as isize, temp.1 as isize));
                }
            }

            Event::WindowEvent{
                event: WindowEvent::MouseInput {state, ..}, ..
            } => {
                console::log_1(&format!("Pressed from mouse input: {:?}", state).into());
                if state == ElementState::Released{
                    pressed = false;
                } else{
                    pressed = true;
                    click_handler.handle_click(current_position);
                }
            }

            Event::RedrawRequested(_) => {
                let paused = *PAUSE.lock().unwrap();
                let barrier_redraw = !click_handler.current_blob.is_empty();
                let mut output_changed = OUTPUT_CHANGED.lock().unwrap();
                let mut color_changed = COLOR_CHANGED.lock().unwrap();
                let mut equilibrium_reset = EQUILIBRIUM_RESET.lock().unwrap();

                if *equilibrium_reset{
                    lbm.reset_to_equilibrium(&driver);
                }

                if *output_changed{
                    let current:SummaryStat =  *CURRENT_OUTPUT.lock().unwrap();
                    lbm.set_summary(current);
                }

                if *color_changed{
                    lbm.color_map = *CURRENT_COLOR_MAP.lock().unwrap();
                }

                if barrier_redraw{
                    lbm.draw_shape(&driver, &click_handler.current_blob);
                    click_handler.update();
                }

                let mut viscosity_changed = VISCOSITY_CHANGED.lock().unwrap();
                if *viscosity_changed{
                    console::log_1(&format!("Viscosity_changed: {}", viscosity_changed).into());
                    let omega = 1.0/(3.0 * *VISCOSITY.lock().unwrap() + 0.5);
                    lbm.update_omega_buffer(&driver, omega);
                    *viscosity_changed = false;
                }

                if !paused{
                    let current:u32 =  *COMPUTE_PER_RENDER.lock().unwrap();
                    lbm.iterate(&driver, current as usize);
                }else if *output_changed || barrier_redraw || *color_changed || *equilibrium_reset{
                    lbm.rerender(&driver);
                }
                *output_changed = false;
                *color_changed = false;
                *equilibrium_reset = false;
            }

            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}

enum ClickType{
    Line,
    Erase, 
    Draw, 
    Inactive
}

struct ClickHandler{
    current_type: ClickType,
    line_points: Vec<(isize, isize)>,
    current_blob: Blob,
    undo_stack: Vec<Box<dyn Shape>>,
    undo_number: usize
}

impl ClickHandler{
    
    pub fn new() -> ClickHandler{
        ClickHandler{
            current_type: ClickType::Draw,
            current_blob: Blob { points: HashSet::<(isize, isize, bool)>::new() },
            line_points: Vec::<(isize, isize)>::new(),
            undo_stack: Vec::<Box<dyn Shape>>::new(),
            undo_number: 0,
        }
    }

    pub fn handle_movement(&mut self, location: (isize, isize)){
        match self.current_type {
            ClickType::Erase => self.erase_update(location),
            ClickType::Draw => self.draw_update(location),
            _ => (),
        }  
    }

    pub fn handle_click(&mut self, click_location: (isize, isize)){
        match self.current_type {
            ClickType::Line => self.line_click(click_location),
            ClickType::Erase => self.erase_update(click_location),
            ClickType::Draw => self.draw_update(click_location),
            _ => (),
        }
    }

    pub fn switch_click_type(&mut self, click_type: ClickType){
        self.current_type = click_type;
        self.line_points.clear();
    }

    pub fn update(&mut self){
        self.undo_stack.push(Box::new(self.current_blob.negate()));
        self.current_blob.empty();
    }

    pub fn undo(&mut self) -> Option<Box<dyn Shape>>{
        if self.current_blob.is_empty(){
            return self.undo_stack.pop();
        } else {
            self.current_blob.empty();
        }
        None
    }

    fn line_click(&mut self, click_location: (isize, isize)){
        self.line_points.push(click_location);
        if self.line_points.len() >= 2{
            let shape = line::Line::new(self.line_points[0], 
                                                 self.line_points[1], 
                                                            X_DIM as isize, 
                                                            Y_DIM as isize)
                                                            .unwrap();
            self.current_blob = merge(&vec![&self.current_blob, &shape], X_DIM as usize);
            self.line_points.clear();
        }
    }

    fn draw_update(&mut self, click_location: (isize, isize)){
        let valid_points = Self::valid_points(&click_location).iter().map(|x| (x.0, x.1, true)).collect();
        self.current_blob.add(&valid_points, X_DIM, Y_DIM);
    }

    fn erase_update(&mut self, click_location: (isize, isize)){
        let valid_points = Self::valid_points(&click_location).iter().map(|x| (x.0, x.1, false)).collect();
        self.current_blob.add(&valid_points, X_DIM, Y_DIM);
    }

    fn filter_points(click_location: &(isize, isize)) -> bool{
        click_location.0 < X_DIM as isize && click_location.1 < Y_DIM as isize
    } 

    fn cube(click_location: &(isize, isize)) -> Vec<(isize, isize)>{
        vec![(click_location.0 - 1, click_location.1 - 1), (click_location.0, click_location.1 - 1), (click_location.0 + 1, click_location.1 - 1),
             (click_location.0 - 1, click_location.1), (click_location.0, click_location.1), (click_location.0 + 1, click_location.1),
             (click_location.0 - 1, click_location.1 + 1), (click_location.0, click_location.1 + 1), (click_location.0 + 1, click_location.1 + 1)]
    }

    fn valid_points(point: &(isize, isize)) -> Vec<(isize, isize)>{
        Self::cube(point).into_iter().filter(|x| Self::filter_points(x)).collect()
    }

}

#[wasm_bindgen]
struct WASMInteraction{
}

#[wasm_bindgen]
impl WASMInteraction {

    pub fn set_output(summary_stat:SummaryStat){
        let mut mutex_changer = CURRENT_OUTPUT.lock().unwrap();
        *mutex_changer = summary_stat;
        let mut mutex_changer = OUTPUT_CHANGED.lock().unwrap();
        *mutex_changer = true;
        console::log_1(&"SET OUTPUT".into());
    }

    pub fn set_color_map(color_map:ColorMap){
        let mut mutex_changer = CURRENT_COLOR_MAP.lock().unwrap();
        *mutex_changer = color_map;
        let mut mutex_changer = COLOR_CHANGED.lock().unwrap();
        *mutex_changer = true;
        console::log_1(&"SET COLOR".into());
    }

    pub fn test(){
        console::log_1(&"Hello using web-sys".into());
    }

    pub fn toggle_pause(){
        let mut mutex_changer = PAUSE.lock().unwrap();
        *mutex_changer = !*mutex_changer;
    }

    pub fn update_compute_rate(rate: u32){
        let mut mutex_changer = COMPUTE_PER_RENDER.lock().unwrap();
        *mutex_changer = rate;
        console::log_1(&format!("{}", rate).into());
    }

    pub fn update_viscosity(viscosity: f32){
        let mut mutex_changer = VISCOSITY.lock().unwrap();
        *mutex_changer = viscosity;
        let mut mutex_changer = VISCOSITY_CHANGED.lock().unwrap();
        *mutex_changer = true;
        console::log_1(&format!("{}", viscosity).into());
    }

    pub fn reset_to_equilibrium(){
        let mut mutex_changer = EQUILIBRIUM_RESET.lock().unwrap();
        *mutex_changer = true;
    }
}


#[wasm_bindgen]
pub fn run() {
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
                        .with_inner_size(PhysicalSize{width: X_DIM as f32, height: Y_DIM as f32})
                        .build(&event_loop).unwrap();
    use winit::platform::web::WindowExtWebSys;
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
