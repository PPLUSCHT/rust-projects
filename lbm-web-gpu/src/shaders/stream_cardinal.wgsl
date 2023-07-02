struct Dimensions{
    row: u32,
    col: u32,
}

@group(0) @binding(0) var<uniform> dimensions: Dimensions;
@group(0) @binding(1) var<storage,read_write> barrier: array<u32>;

@group(1) @binding(0) var<storage, read_write> n: array<f32>;
@group(1) @binding(1) var<storage, read_write> e: array<f32>;
@group(1) @binding(2) var<storage, read_write> s: array<f32>;
@group(1) @binding(3) var<storage, read_write> w: array<f32>;

@group(2) @binding(0) var<storage, read_write> post_n: array<f32>;
@group(2) @binding(1) var<storage, read_write> post_e: array<f32>;
@group(2) @binding(2) var<storage, read_write> post_s: array<f32>;
@group(2) @binding(3) var<storage, read_write> post_w: array<f32>;

fn index(x: u32, y:u32) -> u32{
   return x + y * dimensions.row;
}

fn n_index(index: u32) -> u32{
   return index - dimensions.row;
}

fn w_index(index: u32) -> u32{
   return index - 1u;
}

fn e_index(index: u32) -> u32{
   return index + 1u;
}

fn s_index(index: u32) -> u32{
   return index + dimensions.row;
}

@compute
@workgroup_size(256,1,1)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {

    let index = global_invocation_id.x;

    if (barrier[index] == 1u){
        return;
    }

    if(index > dimensions.row * dimensions.col - 1u){
        return;
    }

    if(index % dimensions.row == 0u){
        return;
    }

    if(index / dimensions.row >= dimensions.col - 1u){
        return;
    }

    let n_index = n_index(index);
    let s_index = s_index(index);
    let e_index = e_index(index);
    let w_index = w_index(index);

    //update n
    if(barrier[s_index] == 1u){
        post_n[index] = s[index];
    } else{
        post_n[index] = n[s_index];
    }

    //update s
    if(barrier[n_index] == 1u){
        post_s[index] = n[index];
    } else{
        post_s[index] = s[n_index];
    }

    //update w
    if(barrier[e_index] == 1u){
        post_w[index] = e[index];
    } else{
        post_w[index] = w[e_index];
    }

    //update e
    if(barrier[w_index] == 1u){
        post_e[index] = w[index];
    } else{
        post_e[index] = e[w_index];
    }

}