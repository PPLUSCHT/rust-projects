struct Dimensions{
    row: u32,
    col: u32,
}

@group(0) @binding(0) var<uniform> omega: f32;
@group(0) @binding(1) var<uniform> dimensions: Dimensions;

@group(1) @binding(0) var<storage, read_write> nNW: array<f32>;
@group(1) @binding(1) var<storage, read_write> nNE: array<f32>;
@group(1) @binding(2) var<storage, read_write> nSE: array<f32>;
@group(1) @binding(3) var<storage, read_write> nSW: array<f32>;

@group(2) @binding(0) var<storage, read_write> nN: array<f32>;
@group(2) @binding(1) var<storage, read_write> nE: array<f32>;
@group(2) @binding(2) var<storage, read_write> nS: array<f32>;
@group(2) @binding(3) var<storage, read_write> nW: array<f32>;

@group(3) @binding(0) var<storage, read_write> n0: array<f32>;
@group(3) @binding(1) var<storage, read_write> curl: array<f32>;

@compute
@workgroup_size(256)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {

    let i = global_invocation_id.x;

    if(i > dimensions.row * dimensions.col - 1u){
        return;
    }

	let thisrho = n0[i] + nN[i] + nS[i] + nE[i] + nW[i] + nNW[i] + nNE[i] + nSW[i] + nSE[i];
	let thisux = (nE[i] + nNE[i] + nSE[i] - nW[i] - nNW[i] - nSW[i]) / thisrho;
	let thisuy = (nN[i] + nNE[i] + nNW[i] - nS[i] - nSE[i] - nSW[i]) / thisrho;
	let one9thrho = 1.0/9.0 * thisrho;		// pre-compute a bunch of stuff for optimization
	let one36thrho = 1.0/36.0 * thisrho;
	let ux3 = 3.0 * thisux;
	let uy3 = 3.0 * thisuy;
	let ux2 = thisux * thisux;
	let uy2 = thisuy * thisuy;
	let uxuy2 = 2.0 * thisux * thisuy;
	let u2 = ux2 + uy2;
	let u215 = 1.5 * u2;
	n0[i]  += omega * (4.0/9.0*thisrho * (1.0                        - u215) - n0[i]);
	nE[i]  += omega * (   one9thrho * (1.0 + ux3       + 4.5*ux2        - u215) - nE[i]);
	nW[i]  += omega * (   one9thrho * (1.0 - ux3       + 4.5*ux2        - u215) - nW[i]);
	nN[i]  += omega * (   one9thrho * (1.0 + uy3       + 4.5*uy2        - u215) - nN[i]);
	nS[i]  += omega * (   one9thrho * (1.0 - uy3       + 4.5*uy2        - u215) - nS[i]);
	nNE[i] += omega * (  one36thrho * (1.0 + ux3 + uy3 + 4.5*(u2+uxuy2) - u215) - nNE[i]);
	nSE[i] += omega * (  one36thrho * (1.0 + ux3 - uy3 + 4.5*(u2-uxuy2) - u215) - nSE[i]);
	nNW[i] += omega * (  one36thrho * (1.0 - ux3 + uy3 + 4.5*(u2-uxuy2) - u215) - nNW[i]);
	nSW[i] += omega * (  one36thrho * (1.0 - ux3 - uy3 + 4.5*(u2+uxuy2) - u215) - nSW[i]);
}
