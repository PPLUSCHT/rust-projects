@group(0) @binding(0) var<storage, read_write> colors: array<f32>;

@group(1) @binding(0) var<storage, read_write> origin: array<f32>;
@group(1) @binding(1) var<storage, read_write> output: array<f32>;

@compute
@workgroup_size(256)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>){
    let total = arrayLength(&colors);
    if (global_invocation_id.x >= total) {
        return;
    }
    colors[global_invocation_id.x] = clamp(output[global_invocation_id.x], -1.0, 1.0);
}