
// NOTE: Both entry functions are called `main`.
//       Because they are marked as entry points,
//       this is ok. They can have different names
//       as well.

// =============================
// Vertex Shader

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] color: vec3<f32>;
};

struct VertexOutput {
    // Builtin attribute tells WGPU that this is the value
    // we want to use as the vertex's clip coordinates.
    // This is analogous to GLSL's gl_Position variable.
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] color: vec3<f32>;
};

[[stage(vertex)]]
fn main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

// =============================
// Fragment Shader

// NOTE: The [[location(0)]] bit tells WGPU to store the
//       vec4 value returned by this function in the first
//       color target.

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
