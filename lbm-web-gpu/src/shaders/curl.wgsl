struct Dimensions{
    row: u32,
    col: u32,
}

@group(0) @binding(0) var<uniform> dimensions: Dimensions;

@group(1) @binding(0) var<storage, read_write> nw: array<f32>;
@group(1) @binding(1) var<storage, read_write> ne: array<f32>;
@group(1) @binding(2) var<storage, read_write> se: array<f32>;
@group(1) @binding(3) var<storage, read_write> sw: array<f32>;

@group(2) @binding(0) var<storage, read_write> n: array<f32>;
@group(2) @binding(1) var<storage, read_write> e: array<f32>;
@group(2) @binding(2) var<storage, read_write> s: array<f32>;
@group(2) @binding(3) var<storage, read_write> w: array<f32>;

@group(3) @binding(0) var<storage, read_write> origin: array<f32>;
@group(3) @binding(1) var<storage, read_write> curl: array<f32>;

fn index(x: u32, y:u32) -> u32{
   return x + y * dimensions.row;
}

fn nw_index(index: u32) -> u32{
   return index - 1u - dimensions.row;
}

fn n_index(index: u32) -> u32{
   return index - dimensions.row;
}

fn ne_index(index: u32) -> u32{
   return index + 1u - dimensions.row;
}

fn w_index(index: u32) -> u32{
   return index - 1u;
}

fn e_index(index: u32) -> u32{
   return index + 1u;
}

fn sw_index(index: u32) -> u32{
   return index - 1u + dimensions.row;
}

fn s_index(index: u32) -> u32{
   return index + dimensions.row;
}

fn se_index(index: u32) -> u32{
   return index + 1u + dimensions.row; 
}

fn rho(i: u32) -> f32{
    return nw[i] + n[i] + ne[i] +
    w[i] + origin[i] + e[i] +
    sw[i] + s[i] + se[i];
}

fn density_x(i: u32) -> f32{
    return e[i] + se[i] + ne[i] - (w[i] + nw[i] + sw[i]);
}

fn density_y(i: u32) -> f32{
    return n[i] + ne[i] + nw[i] - (s[i] + sw[i] + se[i]);
}

@compute
@workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {

   let index = global_invocation_id.x;

    if(index > dimensions.row * dimensions.col - 1u){
        return;
    }

    if(index % dimensions.row == 0u){
        return;
    }

    if(index / dimensions.row >= dimensions.col - 1u){
        return;
    } 

   curl[index] = (density_y(index + 1u) - density_y(index - 1u) - density_x(n_index(index)) + density_x(s_index(index)))/rho(index);
}