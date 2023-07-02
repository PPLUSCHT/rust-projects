struct Dimensions{
    row: u32,
    col: u32,
}

struct VertexOutput{
    @builtin(position) pos: vec4<f32>,
    @location(0) instance_index: u32,
}

fn calc_index(i: u32) -> vec2<f32>{
    return vec2<f32>(2.0*f32(i % dimensions.row)/f32(dimensions.row), -2.0*f32(i / dimensions.row)/f32(dimensions.col)) ;
}

@group(0) @binding(0) var<storage, read_write> color_index: array<f32>;

@group(1) @binding(0) var<uniform> dimensions: Dimensions;
@group(1) @binding(1) var<storage,read_write> barrier: array<u32>;

@vertex
fn vs_main(@location(0) ver: vec2<f32>, @builtin(instance_index) ins: u32) -> VertexOutput {
    var out: VertexOutput;
    out.pos = vec4<f32>(calc_index(ins) + ver, 0.0, 1.0);
    out.instance_index = ins;
    return out;
}

struct FragmentInput{
    @location(0) instance_index: u32,
}

@fragment
fn fs_main(f: FragmentInput) -> @location(0) vec4<f32> {
    let color = color_index[f.instance_index];
    if(barrier[f.instance_index] == 1u){
        return vec4<f32>(0.0, 1.0, 0.0, 1.0);
    }
    if(color < 0.0){
        return vec4<f32>(0.0, 0.0, abs(color), 1.0);
    }
    return vec4<f32>(color, 0.0, 0.0, 1.0);
}