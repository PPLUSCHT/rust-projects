struct Dimensions{
    row: u32;
    col: u32;
}

@group(0) @binding(0) var<uniform> dimensions: Dimensions;
@group(0) @binding(0) var<storage, read> barrier: array<bool>;

@group(1) @binding(0) var<storage, read> nw: array<f32>;
@group(1) @binding(1) var<storage, read> n: array<f32>;
@group(1) @binding(2) var<storage, read> ne: array<f32>;
@group(1) @binding(3) var<storage, read> w: array<f32>;
@group(1) @binding(5) var<storage, read> e: array<f32>;
@group(1) @binding(6) var<storage, read> sw: array<f32>;
@group(1) @binding(7) var<storage, read> s: array<f32>;
@group(1) @binding(8) var<storage, read> se: array<f32>;

@group(2) @binding(0) var<storage, read_write> post_nw: array<f32>;
@group(2) @binding(1) var<storage, read_write> post_n: array<f32>;
@group(2) @binding(2) var<storage, read_write> post_ne: array<f32>;
@group(2) @binding(3) var<storage, read_write> post_w: array<f32>;
@group(2) @binding(5) var<storage, read_write> post_e: array<f32>;
@group(2) @binding(6) var<storage, read_write> post_sw: array<f32>;
@group(2) @binding(7) var<storage, read_write> post_s: array<f32>;
@group(2) @binding(8) var<storage, read_write> post_se: array<f32>;


fn index(x: u32, y:u32) -> u32{
   x + y * dimensions.row
}

fn nw_index(index: u32) -> u32{
   index - 1 - dimensions.row
}

fn n_index(index: u32) -> u32{
   index - dimensions.row
}

fn ne_index(index: u32) -> u32{
   index + 1 - dimensions.row
}

fn w_index(index: u32) -> u32{
   index - 1
}

fn e_index(index: u32) -> u32{
   index + 1
}

fn sw_index(index: u32) -> u32{
   index - 1 + dimensions.row
}

fn s_index(x: u32, y:u32) -> u32{
   index + dimensions.row
}

fn se_index(x: u32, y:u32) -> u32{
   index + 1 + dimensions.row  
}

@compute
@workgroup_size(8,8,1)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {

    let index = index(global_invocation_id.x, global_invocation_id.y);

    if (x_index >= dimensions.x_index) || (y_index >= dimensions.y_index || barrier[index]
        || global_invocation_id.x < 1 || global_invocation_id.x > dimensions.row - 2 ||
        || global_invocation_id.y < 1 || global_invocation_id.y > dimensions.col - 2){
        return;
    }

    let nw_index = nw_index(index);
    if(barrier[nw_index]){
        post_se[index] = nw[index];
    } else{
        post_nw[nw_index] = nw[index];
    }

    let n_index = n_index(index);
    if(barrier[n_index]){
        post_s[index] = n[index];
    } else{
        post_n[n_index] = n[index];
    }

    let ne_index = ne_index(index);
    if(barrier[ne_index]){
        post_sw[index] = ne[index];
    } else{
        post_ne[ne_index] = ne[index];
    }

    let w_index = w_index(index);
    if(barrier[w_index]){
        post_e[index] = w[index];
    } else{
        post_w[w_index] = w[index];
    }

    let e_index = e_index(index);
    if(barrier[e_index]){
        post_w[index] = e[index];
    } else{
        post_e[e_index] = e[index];
    }

    let sw_index = sw_index(index);
    if(barrier[sw_index]){
        post_ne[index] = sw[index];
    } else{
        post_sw[sw_index] = sw[index];
    }

    let s_index = s_index(index);
    if(barrier[s_index]){
        post_n[index] = s[index];
    } else{
        post_s[s_index] = s[index];
    }

    let se_index = se_index(index);
    if(barrier[se_index]){
        post_nw[index] = se[index];
    } else{
        post_se[se_index] = se[index];
    }
}