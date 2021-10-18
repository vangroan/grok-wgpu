
// NOTE: Both entry functions are called `main`.
//       Because they are marked as entry points,
//       this is ok. They can have different names
//       as well.

// =============================
// Vertex Shader

struct VertexOutput {
    // Builtin attribute tells WGPU that this is the value
    // we want to use as the vertex's clip coordinates.
    // This is analogous to GLSL's gl_Position variable.
    [[builtin(position)]] clip_position: vec4<f32>;
};

[[stage(vertex)]]
fn main(
    [[builtin(vertex_index)]] in_vertex_index: u32
) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(1 - i32(in_vertex_index)) * 0.5;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    return out;
}

// =============================
// Fragment Shader

// NOTE: The [[location(0)]] bit tells WGPU to store the
//       vec4 value returned by this function in the first
//       color target.

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(0.3, 0.2, 0.1, 1.0);
}
