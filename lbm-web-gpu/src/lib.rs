use barrier_shapes::{Shape, blob::{Blob, self}, line, curve::Curve, curve_collection::CurveCollection, merge_shapes::{get_points_vector, merge}};
use driver::Driver;
use lbm::ColorMap;
use web_sys::console;
use winit::{event_loop::{EventLoop, ControlFlow}, dpi::PhysicalSize, event::{Event, WindowEvent, ElementState}, window::Window};
use wasm_bindgen::{prelude::*, convert::IntoWasmAbi};

use lazy_static::lazy_static; // 1.4.0
use std::{sync::{Mutex, atomic::AtomicBool}, collections::{HashSet, HashMap, btree_set::Difference}, time::Duration, mem};

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
    static ref CLICK_TYPE: Mutex<ClickType> = Mutex::new(ClickType::Inactive);
    static ref CLICK_TYPE_CHANGED: Mutex<bool> = Mutex::new(true);
    static ref UNDO_CHANGED: Mutex<bool> = Mutex::new(false);
    static ref UNDO_COUNT: Mutex<usize> = Mutex::new(0);
}

use crate::{lbm::SummaryStat, barrier_shapes::line::Line};

pub mod driver;
pub mod barrier_shapes;
pub mod lbm;

const X_DIM:u32 = 2560;
const Y_DIM:u32 = 1440;
const STRETCH: f32 = 1.5;
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
                let temp:(i32, i32) = position.into();
                current_position = ClickHandler::validate_click(((temp.0 as f32/1.5) as isize, (temp.1 as f32/1.5) as isize));
                if pressed{
                    click_handler.handle_movement(current_position);
                }
            }

            Event::WindowEvent{
                event: WindowEvent::MouseInput {state, ..}, ..
            } => {
                let mut click_type_changed = CLICK_TYPE_CHANGED.lock().unwrap();
                if *click_type_changed{
                    let t = CLICK_TYPE.lock().unwrap();
                    click_handler.current_type = *t;
                    *click_type_changed = false;
                }
                if state == ElementState::Released{
                    pressed = false;
                    click_handler.handle_release();
                } else{
                    pressed = true;
                    click_handler.handle_click(current_position);
                }
            }

            Event::RedrawRequested(_) => {
                let paused = *PAUSE.lock().unwrap();
                let mut barrier_redraw = !click_handler.current_blob.is_empty() || !click_handler.current_curve.is_empty();
                let mut output_changed = OUTPUT_CHANGED.lock().unwrap();
                let mut color_changed = COLOR_CHANGED.lock().unwrap();
                let mut equilibrium_reset = EQUILIBRIUM_RESET.lock().unwrap();
                let mut undo_changed = UNDO_CHANGED.lock().unwrap();

                if *undo_changed{
                    let mut undo_count = UNDO_COUNT.lock().unwrap();
                    let mut undo_blob = Blob::new_empty();
                    for i in 0..*undo_count{
                        // click_handler.test_undo();
                        match click_handler.undo() {
                            Some(u) => undo_blob.join(&*u),
                            None => {},
                        }
                    }
                    if !undo_blob.is_empty(){
                        lbm.draw_shape(&driver, &undo_blob);
                    }
                    click_handler.empty_all();
                    barrier_redraw = false;
                    *undo_count = 0;
                }

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
                    click_handler.current_blob.join(&click_handler.current_curve);
                    lbm.draw_shape(&driver, &click_handler.current_blob);
                    click_handler.update(pressed, current_position);
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
                }else if *output_changed || barrier_redraw || *color_changed || *equilibrium_reset || *undo_changed{
                    lbm.rerender(&driver);
                }
                *undo_changed = false;
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

#[wasm_bindgen]
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ClickType{
    Line,
    Erase, 
    Draw, 
    Inactive
}

struct ClickHandler{
    current_type: ClickType,
    line_points: Vec<(isize, isize)>,
    current_blob: Blob,
    current_curve: Curve,
    contiguous_curve: CurveCollection,
    undo_stack: Vec<Box<dyn Shape>>,
    undo_number: usize,
    history: HashMap<(isize, isize), Vec<bool>>
}

impl ClickHandler{
    
    pub fn new() -> ClickHandler{
        ClickHandler{
            current_type: ClickType::Draw,
            current_blob: Blob { points: HashSet::<(isize, isize, bool)>::new() },
            line_points: Vec::<(isize, isize)>::new(),
            undo_stack: Vec::<Box<dyn Shape>>::new(),
            undo_number: 0,
            current_curve: Curve::new(),
            contiguous_curve: CurveCollection::new(),
            history: HashMap::<(isize, isize), Vec<bool>>::new(),
        }
    }

    pub fn handle_movement(&mut self, location: (isize, isize)){
        match self.current_type {
            ClickType::Erase => self.current_curve.erase_segment(location, X_DIM as isize, Y_DIM as isize),
            ClickType::Draw => self.current_curve.add_segment(location, X_DIM as isize, Y_DIM as isize),
            _ => (),
        }  
    }

    pub fn handle_release(&mut self){
        match self.current_type {
            ClickType::Erase => self.release(),
            ClickType::Draw => self.release(),
            _ => (),
        }
    }

    fn release(&mut self){
        //Join current curve to blob so it will be rendered
        self.current_blob.join(&self.current_curve);

        //Add current to contiguous curve
        let mut temp = Curve::new();
        mem::swap(&mut self.current_curve, &mut temp);
        self.contiguous_curve.add_curve(temp);

        //Add contiguous curve to history
        let mut temp = CurveCollection::new();
        mem::swap(&mut self.contiguous_curve, &mut temp);
        self.add_to_history(Box::new(temp));
    }

    pub fn empty_all(&mut self){
        self.current_blob.empty();
        self.current_curve.empty();
    }

    pub fn handle_click(&mut self, click_location: (isize, isize)){
        match self.current_type {
            ClickType::Line => self.line_click(click_location),
            ClickType::Erase => self.current_curve.erase_segment(click_location, X_DIM as isize, Y_DIM as isize),
            ClickType::Draw => self.current_curve.add_segment(click_location, X_DIM as isize, Y_DIM as isize),
            _ => (),
        }
    }

    pub fn switch_click_type(&mut self, click_type: ClickType){
        self.current_type = click_type;
        self.line_points.clear();
    }

    pub fn update(&mut self, pressed: bool, location: (isize, isize)){
        self.current_blob.empty();
        if pressed && *CLICK_TYPE.lock().unwrap() == ClickType::Draw{
            self.draw_update(location);
        }
        if pressed && *CLICK_TYPE.lock().unwrap() == ClickType::Erase{
            self.erase_update(location);
        }
    }

    pub fn undo(&mut self) -> Option<Box<dyn Shape>>{
        if self.current_blob.is_empty() && self.current_curve.is_empty(){
            match self.undo_stack.pop() {
                Some(s) => 
                return Some(self.remove_shape(&*s)),
                None => return None,
            };
        } else {
            console::log_1(&"This is where undo is empty".into());
            self.current_blob.empty();
            self.current_curve.empty();
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
            self.current_blob.join(&shape);
            self.add_to_history(Box::new(shape));
            self.line_points.clear();
        }
    }

    fn draw_update(&mut self, click_location: (isize, isize)){
        let mut temp = Curve::new();
        mem::swap(&mut self.current_curve, &mut temp);
        self.contiguous_curve.add_curve(temp);
        self.current_curve.add_segment(click_location, X_DIM as isize, Y_DIM as isize);
    }

    fn erase_update(&mut self, click_location: (isize, isize)){
        let mut temp = Curve::new();
        mem::swap(&mut self.current_curve, &mut temp);
        self.contiguous_curve.add_curve(temp);
        self.current_curve.erase_segment(click_location, X_DIM as isize, Y_DIM as isize);
    }

    pub fn validate_click(click_location: (isize, isize)) -> (isize, isize){
        let mut new_click = (0,0);
        if click_location.0 < 0{
            new_click.0 = 0;
        } else if click_location.0 >= X_DIM as isize {
            new_click.0 = X_DIM as isize - 1;
        } else {
            new_click.0 = click_location.0;
        }
        if click_location.1 < 0{
            new_click.1 = 0;
        } else if click_location.1 >= Y_DIM as isize {
            new_click.1 = Y_DIM as isize - 1;
        } else{
            new_click.1 = click_location.1;
        }
        new_click
    }

    fn add_to_history(&mut self, shape: Box<dyn Shape>){
        for i in shape.get_points(){
            match self.history.get_mut(&(i.0, i.1)){
                Some(vec) => {vec.push(i.2)},
                None => {
                    self.history.insert((i.0, i.1), vec![i.2]);
                },
            };
        }
        self.undo_stack.push(shape);
    }

    fn remove_shape(&mut self, shape: &dyn Shape) -> Box<dyn Shape>{
        let mut points = Vec::<(isize, isize, bool)>::new();
        for i in shape.get_points(){
            match self.history.get_mut(&(i.0, i.1)){
                Some(vec) => {
                    vec.pop(); 
                    points.push((i.0, i.1, Self::add_point(vec)));
                    if vec.is_empty(){
                        // console::log_1(&"Removing from history".into());
                        self.history.remove(&(i.0, i.1));
                    }
                },
                None => {
                    points.push((i.0, i.1, false));
                    console::log_1(&"remove weirdness".into());
                },
            };
        }
        if self.history.is_empty(){
            // console::log_1(&"history empty".into());
        }
        points.sort();
        let mut blob = Blob::new(HashSet::new());
        blob.add(&points, X_DIM, Y_DIM);
        Box::new(blob)
    }

    fn add_point(vec: &Vec<bool>) -> bool{
       if vec.is_empty(){
        return false;
       }
       vec.last().unwrap().to_owned()
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

    pub fn set_draw_type(draw_type: ClickType){
        let mut mutex_changer = CLICK_TYPE.lock().unwrap();
        *mutex_changer = draw_type;
        let mut mutex_changer = CLICK_TYPE_CHANGED.lock().unwrap();
        *mutex_changer = true;
        console::log_1(&format!("SET CLICKTYPE {:?}", draw_type).into());
    }

    pub fn set_color_map(color_map:ColorMap){
        let mut mutex_changer = CURRENT_COLOR_MAP.lock().unwrap();
        *mutex_changer = color_map;
        let mut mutex_changer = COLOR_CHANGED.lock().unwrap();
        *mutex_changer = true;
        console::log_1(&"SET COLOR".into());
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

    pub fn undo(){
        let mut mutex_changer = UNDO_COUNT.lock().unwrap();
        *mutex_changer += 1;
        let mut mutex_changer = UNDO_CHANGED.lock().unwrap();
        *mutex_changer = true;
    }
}




#[wasm_bindgen]
pub fn run() {
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
                        .with_inner_size(PhysicalSize{width: X_DIM as f32 * 1.5, height: Y_DIM as f32 * 1.5})
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
