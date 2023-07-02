use std::{mem, borrow::Cow};

use wgpu::{BindGroupDescriptor, util::{DeviceExt, BufferInitDescriptor}, BufferUsages, VertexBufferLayout, vertex_attr_array, BindGroupLayout, CommandEncoder};
use crate::driver::Driver;
pub struct Simulation<const X:u32, const Y:u32>{
    
    pub dimension_bind_group: wgpu::BindGroup,
    pub color_bind_group: wgpu::BindGroup,
    pub corner_bind_groups: Vec<wgpu::BindGroup>,
    pub cardinal_bind_groups: Vec<wgpu::BindGroup>,
    pub collide_params_bind_group: wgpu::BindGroup,
    pub origin_output_bind_group: wgpu::BindGroup,
    pub stream_params_bind_group: wgpu::BindGroup,

    pub vertex_buffer: wgpu::Buffer,

    pub collide_pipeline:wgpu::ComputePipeline,
    pub stream_corner_pipeline: wgpu::ComputePipeline,
    pub stream_cardinal_pipeline: wgpu::ComputePipeline,
    pub render_pipeline: wgpu::RenderPipeline,
    pub color_pipeline: wgpu::ComputePipeline,
    pub curl_pipeline: wgpu::ComputePipeline,

    pub frame_num: usize,
}

impl<const X:u32, const Y:u32> Simulation<X, Y>{

    fn create_dimensions_bg(driver: &Driver, dimensions_buffer: &wgpu::Buffer) -> (wgpu::BindGroupLayout ,wgpu::BindGroup){

        let dimension_bind_group_layout = driver.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { 
            label: None, 
            entries: &[
                wgpu::BindGroupLayoutEntry{
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer{
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new((2 * mem::size_of::<u32>()) as _,),
                    },
                    count: None,
                },
            ],
        });

        let dim_bg = driver.device.create_bind_group(&wgpu::BindGroupDescriptor{
            label: None,
            layout: &dimension_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry{
                    binding: 0,
                    resource: dimensions_buffer.as_entire_binding(),
                }
            ],
        });

        (dimension_bind_group_layout, dim_bg)

    }

    fn create_color_bg(driver: &Driver, color_bind_group_layout:&wgpu::BindGroupLayout) ->  wgpu::BindGroup{

        let x_dim = X.to_owned();
        let y_dim = Y.to_owned();

        let color_buffer = driver.device.create_buffer_init(&BufferInitDescriptor{
            label: None,
            contents: bytemuck::cast_slice(&vec![0.0 as f32; (x_dim as usize * y_dim as usize) as usize]),
            usage: BufferUsages::VERTEX| BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC
        });

        let color_bg = driver.device.create_bind_group(
            &BindGroupDescriptor { 
                label: None, 
                layout: &color_bind_group_layout, 
                entries: &[
                    wgpu::BindGroupEntry{
                        binding: 0,
                        resource: color_buffer.as_entire_binding()
                    }
                ],
            }
        );
        color_bg
    }

    fn create_collide_params_bg(driver: &Driver, dimensions_buffer: &wgpu::Buffer, omega: f32) -> (wgpu::BindGroupLayout ,wgpu::BindGroup){
        
        let collide_params_bind_group_layout =  
            driver.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor 
                { entries: &[
                    wgpu::BindGroupLayoutEntry{
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer{
                            ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: wgpu::BufferSize::new(
                                    (1 * mem::size_of::<f32>()) as _,
                                ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry{
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer{
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                (2 * mem::size_of::<u32>()) as _,),
                        },
                        count: None,
                    },
                ],
                label: None
            });

        let omega_buffer = driver.device.create_buffer_init(&wgpu::util::BufferInitDescriptor
            {
                label: Some("Omega"),
                contents: bytemuck::bytes_of(&omega),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        
        let collide_params_bind_group = driver.device.create_bind_group(
            &wgpu::BindGroupDescriptor { 
                label: Some("Collide Parameters"), 
                layout: &collide_params_bind_group_layout, 
                entries: &[
                    wgpu::BindGroupEntry{
                        binding: 0,
                        resource: omega_buffer.as_entire_binding()
                    },
                    wgpu::BindGroupEntry{
                        binding: 1, 
                        resource: dimensions_buffer.as_entire_binding()
                    }
                ]}
        );

        (collide_params_bind_group_layout, collide_params_bind_group)
    }

    fn create_origin_output_bg(driver: &Driver) -> (wgpu::BindGroupLayout ,wgpu::BindGroup){
        
        let origin_output_bind_group_layout =  
            driver.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor 
                { entries: &[
                    wgpu::BindGroupLayoutEntry{
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer{
                            ty: wgpu::BufferBindingType::Storage { read_only: (false) },
                                has_dynamic_offset: false,
                                min_binding_size: wgpu::BufferSize::new(
                                    (X as usize * Y as usize * mem::size_of::<f32>()) as _,
                                ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry{
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer{
                            ty: wgpu::BufferBindingType::Storage { read_only: (false) },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                (X as usize * Y as usize * mem::size_of::<f32>()) as _,),
                        },
                        count: None,
                    },
                ],
                label: None
            });
        
        let init_state = Self::set_equil(0.1, 0.0, 1.0);

        let origin_buffer = driver.device.create_buffer_init(&BufferInitDescriptor { 
            label: None, 
            contents: bytemuck::cast_slice(&init_state[4]), 
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC
        });

        let output_buffer = driver.device.create_buffer_init(&BufferInitDescriptor { 
            label: None, 
            contents: bytemuck::cast_slice(&vec![0.0; X as usize * Y as usize]), 
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC
        });

        let origin_output_bind_group = driver.device.create_bind_group(
            &wgpu::BindGroupDescriptor { 
                label: Some("Collide Parameters"), 
                layout: &origin_output_bind_group_layout, 
                entries: &[
                    wgpu::BindGroupEntry{
                        binding: 0,
                        resource: origin_buffer.as_entire_binding()
                    },
                    wgpu::BindGroupEntry{
                        binding: 1, 
                        resource: output_buffer.as_entire_binding(),
                    }
                ]}
        );

        (origin_output_bind_group_layout, origin_output_bind_group)
    }

    fn create_data_bgs(driver: &Driver) -> (wgpu::BindGroupLayout, Vec<wgpu::BindGroup>, Vec<wgpu::BindGroup>){
        
        let data_bind_group_layout = 
            driver.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor 
                { entries: &[
                    wgpu::BindGroupLayoutEntry{
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer { 
                            ty: wgpu::BufferBindingType::Storage { read_only: false }, 
                            has_dynamic_offset: false, 
                            min_binding_size: wgpu::BufferSize::new(
                                (X as usize * Y as usize * mem::size_of::<f32>()) as _,), 
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry{
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer { 
                            ty: wgpu::BufferBindingType::Storage { read_only: false }, 
                            has_dynamic_offset: false, 
                            min_binding_size: wgpu::BufferSize::new(
                                (X as usize * Y as usize * mem::size_of::<f32>()) as _,), 
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry{
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer { 
                            ty: wgpu::BufferBindingType::Storage { read_only: false }, 
                            has_dynamic_offset: false, 
                            min_binding_size: wgpu::BufferSize::new(
                                (X as usize * Y as usize * mem::size_of::<f32>()) as _,), 
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry{
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer { 
                            ty: wgpu::BufferBindingType::Storage { read_only: false }, 
                            has_dynamic_offset: false, 
                            min_binding_size: wgpu::BufferSize::new(
                                (X as usize * Y as usize * mem::size_of::<f32>()) as _,), 
                        },
                        count: None,
                    },
                ],
                label: None,
            });

        let init_state = Self::set_equil(0.1, 0.0, 1.0);
        
        let mut odd_data_buffers = Vec::<wgpu::Buffer>::with_capacity(9);
        let mut even_data_buffers = Vec::<wgpu::Buffer>::with_capacity(9);

        let mut corner_bind_groups = Vec::<wgpu::BindGroup>::with_capacity(2);
        let mut cardinal_bind_groups = Vec::<wgpu::BindGroup>::with_capacity(2);

        for i in 0..9{
            odd_data_buffers.push(
                driver.device.create_buffer_init(&wgpu::util::BufferInitDescriptor
                {
                    label: Some(&format!("{}, Odd", Self::int_to_direction(i))),
                    contents: bytemuck::cast_slice(&init_state[i]),
                    usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
                })
            );

            even_data_buffers.push(
                driver.device.create_buffer_init(&wgpu::util::BufferInitDescriptor
                    {
                        label: Some(&format!("{}, Even", Self::int_to_direction(i))),
                        contents: bytemuck::cast_slice(&init_state[i]),
                        usage: wgpu::BufferUsages::VERTEX
                        | wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC,
                    })
            );
        }
        
        corner_bind_groups.push(
            driver.device.create_bind_group(
                &wgpu::BindGroupDescriptor{
                    layout: &data_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: odd_data_buffers[0].as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: odd_data_buffers[2].as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: odd_data_buffers[8].as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: odd_data_buffers[6].as_entire_binding(),
                        }
                    ],
                    label: Some("Odd corner binding")
                } 
            )    
        );

        corner_bind_groups.push(
            driver.device.create_bind_group(
                &wgpu::BindGroupDescriptor{
                    layout: &data_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: even_data_buffers[0].as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: even_data_buffers[2].as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: even_data_buffers[8].as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: even_data_buffers[6].as_entire_binding(),
                        }
                    ],
                    label: Some("Even corner binding")
                } 
            )    
        );

        cardinal_bind_groups.push(
            driver.device.create_bind_group(
                &wgpu::BindGroupDescriptor{
                    layout: &data_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: odd_data_buffers[1].as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: odd_data_buffers[5].as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: odd_data_buffers[7].as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: odd_data_buffers[3].as_entire_binding(),
                        }
                    ],
                    label: Some("Odd cardinal binding")
                } 
            )    
        );

        cardinal_bind_groups.push(
            driver.device.create_bind_group(
                &wgpu::BindGroupDescriptor{
                    layout: &data_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: even_data_buffers[1].as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: even_data_buffers[5].as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: even_data_buffers[7].as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: even_data_buffers[3].as_entire_binding(),
                        }
                    ],
                    label: Some("Even cardinal binding")
                } 
            )    
        );

        (data_bind_group_layout, corner_bind_groups, cardinal_bind_groups)

    }

    fn create_stream_params_bg(driver: &Driver, dimensions_buffer: &wgpu::Buffer, barrier_buffer: wgpu::Buffer) -> (wgpu::BindGroupLayout ,wgpu::BindGroup){

        let stream_params_bind_group_layout = 
            driver.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor 
                { entries: &[
                    wgpu::BindGroupLayoutEntry{
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer{
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                (2 * mem::size_of::<u32>()) as _,),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry{
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer{
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: wgpu::BufferSize::new(
                                    (Y as usize * X as usize * mem::size_of::<bool>()) as _,
                                ),
                        },
                        count: None,
                    },
                ],
                label: None
            });
        
        let stream_params_bind_group = driver.device.create_bind_group(
            &wgpu::BindGroupDescriptor { 
                label: Some("Stream Parameters"), 
                layout: &stream_params_bind_group_layout, 
                entries: &[
                    wgpu::BindGroupEntry{
                        binding: 0,
                        resource: dimensions_buffer.as_entire_binding()
                    },
                    wgpu::BindGroupEntry{
                        binding: 1,
                        resource: barrier_buffer.as_entire_binding()
                    }
                ] 
            }
        );

        (stream_params_bind_group_layout, stream_params_bind_group)

    }

    fn create_vertex_buffer(driver: &Driver) -> wgpu::Buffer{
       driver.device.create_buffer_init(&BufferInitDescriptor{
            label: None,
            contents: bytemuck::cast_slice(&[-1.0, 1.0, -1.0, 1.0 - 2.0/Y as f32, -1.0 + 2.0/X as f32, 1.0 - 2.0/Y as f32, -1.0, 1.0, -1.0 + 2.0/X as f32, 1.0, -1.0 + 2.0/X as f32, 1.0 - 2.0/Y as f32]),
            usage: BufferUsages::VERTEX
        })
    }

    fn create_collide_pipeline(driver: &Driver, 
                               params: &wgpu::BindGroupLayout, 
                               data: &wgpu::BindGroupLayout, 
                               matrix: &wgpu::BindGroupLayout) 
                               -> wgpu::ComputePipeline{

        let collide_shader = driver.device.create_shader_module(wgpu::ShaderModuleDescriptor{
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/collide.wgsl"))),
        });
        
        let collide_pipeline_layout = 
            driver.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { 
                label: Some("Collide"), 
                bind_group_layouts: &[params, data, data, matrix], 
                push_constant_ranges: &[]
        });

        driver.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { 
            label: None, 
            layout: Some(&collide_pipeline_layout), 
            module: &collide_shader, 
            entry_point: "main" 
        })
    }

    fn create_stream_layout(driver: &Driver, 
                            params: &wgpu::BindGroupLayout, 
                            data: &wgpu::BindGroupLayout)
                            -> wgpu::PipelineLayout{
        driver.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { 
            label: Some("Stream"), 
            bind_group_layouts: &[params, data, data], 
            push_constant_ranges: &[]
        })
    }

    fn create_stream_corners_pipeline(driver: &Driver,
                                      layout: &wgpu::PipelineLayout)
                                      -> wgpu::ComputePipeline{
        let stream_corners_shader = driver.device.create_shader_module(wgpu::ShaderModuleDescriptor{
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/stream_corners.wgsl"))),
        });

        driver.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { 
            label: None, 
            layout: Some(layout), 
            module: &stream_corners_shader, 
            entry_point: "main" 
        })
    }

    fn create_stream_cardinal_pipeline(driver: &Driver,
                                      layout: &wgpu::PipelineLayout)
                                      -> wgpu::ComputePipeline{
        
        let stream_cardinal_shader = driver.device.create_shader_module(wgpu::ShaderModuleDescriptor{
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/stream_cardinal.wgsl"))),
        });

        driver.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { 
            label: None, 
            layout: Some(layout), 
            module: &stream_cardinal_shader, 
            entry_point: "main" 
        })
    }

    fn create_render_pipeline(driver: &Driver,
                              colors: &BindGroupLayout,
                              stream_params: &BindGroupLayout
                            ) -> wgpu::RenderPipeline{

        let render_shader = driver.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/render.wgsl"))),
        });

        let render_pipeline_layout = driver.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&colors, &stream_params],
            push_constant_ranges: &[],
        });

        let swapchain_capabilities = driver.surface.get_capabilities(&driver.adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        driver.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &render_shader,
                entry_point: "vs_main",
                buffers: &[VertexBufferLayout{
                    array_stride: 4 * 2,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &vertex_attr_array![0 => Float32x2],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &render_shader,
                entry_point: "fs_main",
                targets: &[Some(swapchain_format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        })
    }

    fn create_curl_pipeline(driver: &Driver, 
        params: &wgpu::BindGroupLayout, 
        data: &wgpu::BindGroupLayout, 
        origin_output: &wgpu::BindGroupLayout) 
        -> wgpu::ComputePipeline{

        let curl_shader = driver.device.create_shader_module(wgpu::ShaderModuleDescriptor{
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/curl.wgsl"))),
        });

        let curl_pipeline_layout = 
            driver.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { 
            label: Some("Collide"), 
            bind_group_layouts: &[params, data, data, origin_output], 
            push_constant_ranges: &[]
        });

        driver.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { 
            label: None, 
            layout: Some(&curl_pipeline_layout), 
            module: &curl_shader, 
            entry_point: "main" 
        })
    }

    fn create_color_pipeline(driver: &Driver, 
        color: &wgpu::BindGroupLayout,  
        origin_output: &wgpu::BindGroupLayout) 
        -> wgpu::ComputePipeline{

        let color_shader = driver.device.create_shader_module(wgpu::ShaderModuleDescriptor{
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/color_map.wgsl"))),
        });

        let color_pipeline_layout = 
            driver.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { 
            label: Some("Color"), 
            bind_group_layouts: &[color, origin_output], 
            push_constant_ranges: &[]
        });

        driver.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { 
            label: None, 
            layout: Some(&color_pipeline_layout), 
            module: &color_shader, 
            entry_point: "main" 
        })
    }


    pub async fn new(driver: &Driver, omega: f32) -> Simulation<X,Y>{
        
        //create common resources
        let init_state = Self::set_equil(0.1, 0.0, 1.0);
        let barriers = Self::init_barrier(&driver.device);

        let dimensions_buffer = driver.device.create_buffer_init(&BufferInitDescriptor{
            label: None,
            contents: bytemuck::cast_slice(&[X,Y]),
            usage: BufferUsages::COPY_DST| BufferUsages::UNIFORM
        });

        let matrix_bind_group_layout = driver.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { 
            label: None, 
            entries: &[
                wgpu::BindGroupLayoutEntry{
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer{
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new((mem::size_of::<f32>() * X as usize * Y as usize) as _,),
                    },
                    count: None,
                },
            ],
        });

        //create all bindgroups and layouts
        let dimensions_tuple = Self::create_dimensions_bg(driver, &dimensions_buffer);
        let color_bg = Self::create_color_bg(driver, &matrix_bind_group_layout);
        let collide_tuple = Self::create_collide_params_bg(driver, &dimensions_buffer, omega);
        let stream_params_tuple = Self::create_stream_params_bg(driver, &dimensions_buffer, barriers);
        let data_tuple = Self::create_data_bgs(driver);
        let origin_output_tuple = Self::create_origin_output_bg(driver);

        //create vertex buffer
        let vertex_buffer = Self::create_vertex_buffer(driver);

        //create pipelines
        let collide_pipeline = Self::create_collide_pipeline(driver, &collide_tuple.0, &data_tuple.0, &origin_output_tuple.0);
        let stream_pipeline_layout = Self::create_stream_layout(driver, &stream_params_tuple.0, &data_tuple.0);
        let stream_corner_pipeline = Self::create_stream_corners_pipeline(driver, &stream_pipeline_layout);
        let stream_cardinal_pipeline = Self::create_stream_cardinal_pipeline(driver, &stream_pipeline_layout);
        let render_pipeline = Self::create_render_pipeline(driver, &matrix_bind_group_layout, &stream_params_tuple.0);
        let color_pipeline = Self::create_color_pipeline(driver, &matrix_bind_group_layout, &origin_output_tuple.0);
        let curl_pipeline = Self::create_curl_pipeline(driver, &dimensions_tuple.0, &data_tuple.0, &origin_output_tuple.0);

        Simulation{
            dimension_bind_group: dimensions_tuple.1,
            color_bind_group: color_bg,
            corner_bind_groups: data_tuple.1,
            cardinal_bind_groups: data_tuple.2,
            collide_params_bind_group: collide_tuple.1,
            origin_output_bind_group: origin_output_tuple.1,
            stream_params_bind_group: stream_params_tuple.1,
            vertex_buffer,
            collide_pipeline,
            stream_corner_pipeline,
            stream_cardinal_pipeline,
            render_pipeline,
            color_pipeline,
            curl_pipeline,
            frame_num: 0,
        }
    }

    pub fn iterate(&mut self, driver: &Driver, compute_steps: usize){
        let mut encoder = driver.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        for i in 0..compute_steps{
            self.compute_step(driver, &mut encoder);
        }
        self.curl(driver, &mut encoder);
        self.color_map(driver, &mut encoder);
        driver.queue.submit(Some(encoder.finish()));
        self.render(driver);
    }

    fn compute_step(&mut self, driver: &Driver, encoder: &mut CommandEncoder){
        self.collide(driver, encoder);
        self.stream_cardinal(driver, encoder);
        self.stream_corner(driver, encoder);
        self.frame_num += 1;
    }

    fn render(&mut self, driver: &Driver) {
        let mut encoder = driver.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let frame = driver.surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture");
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.set_bind_group(0, &self.color_bind_group, &[]);
            rpass.set_bind_group(1, &self.stream_params_bind_group, &[]);
            rpass.draw(0..6, 0..X*Y);
        }
        driver.queue.submit(Some(encoder.finish()));
        frame.present();
    }


    fn collide(&mut self, driver: &Driver, encoder: &mut CommandEncoder){
        let work_group_count = X * Y  as u32;
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        cpass.set_pipeline(&self.collide_pipeline);
        cpass.set_bind_group(0, &self.collide_params_bind_group, &[]);
        cpass.set_bind_group(1, &self.corner_bind_groups[self.frame_num % 2], &[]);
        cpass.set_bind_group(2, &self.cardinal_bind_groups[self.frame_num % 2], &[]);
        cpass.set_bind_group(3, &self.origin_output_bind_group, &[]);
        cpass.dispatch_workgroups(work_group_count, 1, 1);
    }

    fn stream_cardinal(&mut self, driver: &Driver, encoder: &mut CommandEncoder){
        let work_group_count = X * Y  as u32;
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        cpass.set_pipeline(&self.stream_cardinal_pipeline);
        cpass.set_bind_group(0, &self.stream_params_bind_group, &[]);
        cpass.set_bind_group(1, &self.cardinal_bind_groups[self.frame_num % 2], &[]);
        cpass.set_bind_group(2, &self.cardinal_bind_groups[(self.frame_num + 1) % 2], &[]);
        cpass.dispatch_workgroups(work_group_count, 1, 1);
    }
    
    fn stream_corner(&mut self, driver: &Driver, encoder: &mut CommandEncoder){
        let work_group_count = X * Y  as u32;
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        cpass.set_pipeline(&self.stream_corner_pipeline);
        cpass.set_bind_group(0, &self.stream_params_bind_group, &[]);
        cpass.set_bind_group(1, &self.corner_bind_groups[self.frame_num % 2], &[]);
        cpass.set_bind_group(2, &self.corner_bind_groups[(self.frame_num + 1) % 2], &[]);
        cpass.dispatch_workgroups(work_group_count, 1, 1);
    }

    fn curl(&mut self, driver: &Driver, encoder: &mut CommandEncoder){
        let work_group_count = X * Y  as u32;
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        cpass.set_pipeline(&self.curl_pipeline);
        cpass.set_bind_group(0, &self.dimension_bind_group, &[]);
        cpass.set_bind_group(1, &self.corner_bind_groups[(self.frame_num + 1) % 2], &[]);
        cpass.set_bind_group(2, &self.cardinal_bind_groups[(self.frame_num + 1) % 2], &[]);
        cpass.set_bind_group(3, &self.origin_output_bind_group, &[]);
        cpass.dispatch_workgroups(work_group_count, 1, 1);
    }

    fn color_map(&mut self, driver: &Driver, encoder: &mut CommandEncoder){
        let work_group_count = X * Y  as u32;
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        cpass.set_pipeline(&self.color_pipeline);
        cpass.set_bind_group(0, &self.color_bind_group, &[]);
        cpass.set_bind_group(1, &self.origin_output_bind_group, &[]);
        cpass.dispatch_workgroups(work_group_count, 1, 1);
    }

    fn set_equil(mut ux: f32, mut uy: f32, rho: f32) -> Vec<Vec<f32>>{
    
        let mut ux_2 = ux * ux;
        let mut uy_2 = uy * uy;
        let mut u_dot_product = ux_2 + uy_2;
        let mut u_sum_sq_pos = u_dot_product + 2.0 * (ux * uy);
        let mut u_sum_sq_neg = u_dot_product - 2.0 * (ux * uy);
    
        ux *= 3.0;
        uy *= 3.0;
        ux_2 *= 4.5;
        uy_2 *= 4.5;
        u_dot_product *= 1.5;
        u_sum_sq_neg *= 4.5;
        u_sum_sq_pos *= 4.5;
    
        let rho_ninth = rho/9.0_f32;
        let rho_36th = rho/36.0_f32;
    
        let mut vec = Vec::with_capacity(9);
    
        vec.push(vec![rho_36th * (1.0 - ux + uy + u_sum_sq_neg - u_dot_product); X as usize * Y as usize]);
        vec.push(vec![rho_ninth * (1.0 + uy + uy_2 - u_dot_product);  X as usize * Y as usize]);
        vec.push(vec![rho_36th * (1.0 + ux + uy + u_sum_sq_pos - u_dot_product);  X as usize * Y as usize]);
        vec.push(vec![rho_ninth * (1.0 - ux + ux_2 - u_dot_product);  X as usize * Y as usize]);
        // below changed
        vec.push(vec![4.0 * rho_ninth * (1.0 - u_dot_product);  X as usize * Y as usize]);
        vec.push(vec![rho_ninth * (1.0 + ux + ux_2 - u_dot_product);  X as usize * Y as usize]);
        vec.push(vec![rho_36th * (1.0 - ux - uy + u_sum_sq_pos - u_dot_product);  X as usize * Y as usize]);
        vec.push(vec![rho_ninth * (1.0 - uy - uy_2 - u_dot_product);  X as usize * Y as usize]);
        vec.push(vec![rho_36th * (1.0 + ux - uy + u_sum_sq_neg - u_dot_product);  X as usize * Y as usize]);
        vec
        
    }

    fn int_to_direction(i:usize) -> &'static str{
        match i {
            0 => return "Northwest",
            1 => return "North",
            2 => return "Northeast",
            3 => return "West",
            4 => return "Origin",
            5 => return "East",
            6 => return "Southwest",
            7 => return "South",
            8 => return "Southeast",
            _ => return "Out of bounds"
        }
    }

    fn init_barrier(device: &wgpu::Device) -> wgpu::Buffer{
        
        let mut border_vec = vec![0_u32; X as usize * Y as usize];
        let chevron_height = std::cmp::min(Y/5,X/5);
        let start_y = Y/2;
        let start_x = X/3;

        for i in 0..X{
            border_vec[Self::index(i, 0) as usize] = 1;
            border_vec[Self::index(i, Y - 1) as usize] =1;
        }
        for i in 0..chevron_height{
            border_vec[Self::index(start_x - i, start_y + i) as usize] = 1;
            border_vec[Self::index(start_x - i - 1, start_y + i) as usize] = 1;
            border_vec[Self::index(start_x - i - 2, start_y + i) as usize] = 1;

            border_vec[Self::index(start_x - i, start_y - i) as usize] = 1;
            border_vec[Self::index(start_x - i - 1, start_y - i) as usize] = 1;
            border_vec[Self::index(start_x - i - 2, start_y - i) as usize] = 1;
        }

        // for i in 0..chevron_height{
        //     border_vec[Self::index(start_x, start_y + i) as usize] = 1;
        //     border_vec[Self::index(start_x, start_y - i) as usize] = 1;
        // }

        device.create_buffer_init(&wgpu::util::BufferInitDescriptor
            {
                label: Some("Border"),
                contents: bytemuck::cast_slice(&border_vec),
                usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            })
    }

    fn index(x: u32, y:u32) -> u32{
        x + y * X
    }

}