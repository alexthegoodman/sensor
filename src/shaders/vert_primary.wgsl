// // Vertex input structure definition
// struct VertexInput {
//     @location(0) position: vec3<f32>,
//     @location(1) tex_coords: vec2<f32>,
//     @location(2) color: vec4<f32>,  // Receive color from the vertex buffer
// };

// struct VertexOutput {
//     @builtin(position) clip_position: vec4<f32>,
//     @location(0) tex_coords: vec2<f32>,
//     @location(1) color: vec4<f32>,  // Pass color to the fragment shader
// };

// @vertex
// fn vs_main(in: VertexInput) -> VertexOutput {
//     var out: VertexOutput;
//     out.clip_position = vec4(in.position, 1.0);
//     out.tex_coords = in.tex_coords;
//     out.color = in.color;  // Pass color from input to output
//     return out;
// }

struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,  // Receive color from the vertex buffer
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,  // Pass color to the fragment shader
};

@vertex
fn vs_main(
    vertex: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    let pos = vec4<f32>(vertex.position, 1.0);
    out.clip_position = camera.view_proj * pos;
    out.tex_coords = vertex.tex_coords;
    out.color = vertex.color;  // Pass color from input to output
    return out;
}