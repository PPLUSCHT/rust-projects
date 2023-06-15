struct Dimensions{
    row: u32;
    col: u32;
}

@group(0) @binding(0) var<uniform> omega: f32;
@group(0) @binding(0) var<uniform> dimensions: Dimensions;

@group(1) @binding(0) var<storage, read> nw: array<f32>;
@group(1) @binding(1) var<storage, read> n: array<f32>;
@group(1) @binding(2) var<storage, read> ne: array<f32>;
@group(1) @binding(3) var<storage, read> w: array<f32>;
@group(1) @binding(4) var<storage, read> origin: array<f32>;
@group(1) @binding(5) var<storage, read> e: array<f32>;
@group(1) @binding(6) var<storage, read> sw: array<f32>;
@group(1) @binding(7) var<storage, read> s: array<f32>;
@group(1) @binding(8) var<storage, read> se: array<f32>;


fn index(x: u32, y:u32) -> u32{
   x + y * dimensions.row
} 

fn rho(i: u32) -> f34{
    nw[i] + n[i] + ne[i] +
    w[i] + origin[i] + e[i] +
    sw[i] + s[i] + se[i]
}

fn density_x(i: u32) -> f32{
    e + se + ne - 
    (w + nw + sw)
}

fn density_y(i: u32) -> f32{
    n + ne + nw - 
    (s + sw + se)
}

@compute
@workgroup_size(8,8,1)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let x_index = global_invocation_id.x;
    let y_index = global_invocation_id.y;
    if (x_index >= dimensions.x_index) || (y_index >= dimensions.y_index){
        return;
    }

    let index = index(x_index, y_index);
    let rho = rho();
    let rho_ninth = rho/9.0_f32;
    let rho_36th = rho/36.0_f32;

    //needed directional terms
    var ux = density_x(index)/rho;
    var uy = density_y(index)/rho;
    var ux_2 = ux * ux;
    var uy_2 = uy * uy;
    var u_dot_product = ux_2 + uy_2;
    var u_sum_sq_pos = u_dot_product + 2.0 * (ux * uy);
    var u_sum_sq_neg = u_dot_product - 2.0 * (ux * uy);

    ux *= 3.0;
    uy *= 3.0;
    ux_2 *= 4.5;
    uy_2 *= 4.5;
    u_dot_product *= 1.5;
    u_sum_sq_neg *= 4.5;
    u_sum_sq_pos *= 4.5;

    origin[index] += omega * (4.0 * rho_ninth * (1.0 - u_dot_product) - origin[index]);
    e[index] += omega * (rho_ninth * (1.0 + ux + ux_2 - u_dot_product) - e[index]);
    w[index] += omega * (rho_ninth * (1.0 - ux + ux_2 - u_dot_product) - w[index]);
    n[index] += omega * (rho_ninth * (1.0 + uy + uy_2 - u_dot_product) - n[index]);
    s[index] += omega * (rho_ninth * (1.0 - uy - uy_2 - u_dot_product) - s[index]);
    ne[index] += omega * (rho_36th * (1.0 + ux + uy + u_sum_sq_pos - u_dot_product) - ne[index]);
    se[index] += omega * (rho_36th * (1.0 + ux - uy + u_sum_sq_neg - u_dot_product) - se[index]);
    nw[index] += omega * (rho_36th * (1.0 - ux + uy + u_sum_sq_neg - u_dot_product) - nw[index]);
    sw[index] += omega * (rho_36th * (1.0 - ux - uy + u_sum_sq_pos - u_dot_product) - sw[index]);

}