use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            // The `array_stride` defines how wide a vertex is. When the shader
            // goes to read the next vertex, it will skip over `array_stride`
            // number of bytes. In our case, `array_stride` will probably be 24 bytes.
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            // Tells the pipeline how often it should move to the next vertex.
            step_mode: wgpu::VertexStepMode::Vertex,
            // Vertex attributes describe the individual parts of the vertex.
            // Generally this is a 1:1 mapping with a struct's fields, which
            // it is in our case.
            attributes: &[
                wgpu::VertexAttribute {
                    // Defines the offset in bytes that this attribute starts.
                    // The first attribute is usually zero, and any future
                    // attributes are the collective size_of the previous
                    // attributes data.
                    offset: 0,
                    // Tells the shader what location to store this attribute at.
                    // For example `[[location(0)]] x: vec3<f32>` in the vertex
                    // shader would correspond to the position field of the struct,
                    // while `[[location(1)]] x: vec3<f32>` would be the color field.
                    shader_location: 0,
                    // Format tells the shader the shape of the attribute.
                    // `Float32x3` corresponds to vec3<f32> in shader code.
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

// Triangle
#[rustfmt::skip]
const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.0868241,   0.49240386, 0.0], color: [0.5, 0.0, 0.5] }, // A
    Vertex { position: [-0.49513406,  0.06958647, 0.0], color: [0.5, 0.0, 0.5] }, // B
    Vertex { position: [-0.21918549, -0.44939706, 0.0], color: [0.5, 0.0, 0.5] }, // C
    Vertex { position: [ 0.35966998, -0.3473291,  0.0], color: [0.5, 0.0, 0.5] }, // D
    Vertex { position: [ 0.44147372,  0.2347359,  0.0], color: [0.5, 0.0, 0.5] }, // E
];

#[rustfmt::skip]
const INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
    // IMPORTANT: We add 2 bytes padding as wgpu requires buffers to be aligned to 4 bytes.
    /* padding */
    0,
];

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

impl State {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };

        // The adapter is a handle to our actual graphics card.
        // We can use this get information about the graphics
        // card such as its name and what backend the adapter
        // uses. We use this to create our Device and Queue later.
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                // wgpu can pick between low power devices like integrated graphics,
                // or high power consumption like a dedicated card.
                power_preference: wgpu::PowerPreference::HighPerformance,
                // Tells wgpu to find an adapter that can present to the supplied surface.
                // Our window needs to implement raw-window-handle's HasRawWindowHandle
                // trait to create a surface.
                compatible_surface: Some(&surface),
                // Forces wgpu to pick an adapter that will work on all harware.
                // This usually means that the rendering backend will use a
                // "software" system, instead of hardware such as a GPU.
                force_fallback_adapter: false,
            })
            .await
            .expect("failed to create adapter");

        // Requests a connection to a physical device, creating a logical device.
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    // Allows us to specify what extra features we want.
                    // For this simple example, I've decided not to use
                    // any extra features.
                    //
                    // We can get a list of features supported by our
                    // device using `adapter.features()`, or `device.features()`
                    features: wgpu::Features::empty(),
                    // The limits field describes the limit of certain
                    // types of resources we can create. If any requested
                    // limits are beyond the hardware device, creation
                    // will fail.
                    limits: wgpu::Limits::default(),
                    // Debug label for the device.
                    label: Some("Adapter"),
                },
                // trace_path - Can be used for API call tracing,
                //              if that feature is enabled in wgpu-core.
                None,
            )
            .await
            .expect("failed to create device");

        // This will define how the surface creates its underlying `SurfaceTexture`
        let config = wgpu::SurfaceConfiguration {
            // `usage` field describes how the `SurfaceTexture`s
            // will be used. RENDER_ATTACHMENT specifies that the
            // textures will be used to write to the screen.
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            // `format` defines how the `SurfaceTexture`s will be
            // stored on the gpu. Different displays prefer different
            // formats. We use `get_preferred_format` to figure out
            // the best format to use based on the display you're using.
            format: surface.get_preferred_format(&adapter).unwrap(),
            // The width and height in pixels of the SurfaceTexture.
            // This should usually be the width and height of the window.
            //
            // WARNING: Make sure that the width and height of the
            //          `SurfaceTexture` are not 0, as that can
            //          cause the app to crash.
            width: size.width,
            height: size.height,
            // Determines how to sync the surface with the display.
            // The option we picked FIFO, will cap the display rate
            // at the displays framerate. This is essentially VSync
            //  This is also the most optimal mode on mobile.
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        // Render Pipeline
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                // Here we can specify the function name to be called
                // in the shader module.
                entry_point: "main",
                // Tells wgpu what type of vertices we want to pass to
                // the vertex shader. If the vertices are generated in
                // the shader, then this can be left empty.
                buffers: &[Vertex::layout()],
            },
            // Fragment shader is optional. We need it because we're storing
            // color data to the surface.
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "main",
                // The `targets` field tells wgpu what color outputs it should set up.
                // Currently we only need one for the surface. We use the surface's
                // format so that copying to it is easy, and we specify that the
                // blending should just replace old pixel data with new data.
                //
                // We also tell wgpu to write to all colors: red, blue, green, and alpha.
                targets: &[wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                // Means that each three vertices will correspond to one triangle.
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                // The `front_face` and `cull_mode` fields tell wgpu how to
                // determine whether a given triangle is facing forward or not.
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLAMPING
                clamp_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                // This has to do with anti-aliasing.
                alpha_to_coverage_enabled: false,
            },
        });

        // `create_buffer_init` requires the DeviceExt extension trait.
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_indices = INDICES.len() as u32;

        State {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        // Size 0 will crash the app.
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        // remove `todo!()`
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // Will wait for the surface to provide a new
        // SurfaceTexture that we will render to.
        let output = self.surface.get_current_texture()?;

        // We do this because we want to control how the
        // render code interacts with the texture.
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Most modern graphics frameworks expect commands to
        // be stored in a command buffer before being sent
        // to the gpu. The encoder builds a command buffer that
        // we can then send to the gpu.
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            // We need to use the encoder to create a RenderPass.
            // The RenderPass has all the methods to do the actual drawing.
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                // Describe where we are going to draw our color to.
                // We use the TextureView we created earlier to make
                // sure that we render to the screen.
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    // The texture that will receive the resolved output.
                    // This will be the same as view unless multisampling
                    // is enabled. We don't need to specify this, so we
                    // leave it as None.
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // This tells wgpu what to do with the colors on
                        // the screen (specified by `frame.view`).
                        // The `load` field tells wgpu how to handle
                        // colors stored from the previous frame.
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        // The `store` field tells wgpu with we want to
                        // store the rendered results to the Texture behind
                        // our `TextureView` (in this case it's the `SurfaceTexture`).
                        // We use true as we do want to store our render results.
                        // There are cases when you wouldn't want to.
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);

            // Vertex buffer must be set, otherwise program will crash.
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            // The method name is `set_index_buffer` not `set_index_buffers`.
            // We can only have one index buffer set at a time.
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            // This is where [[builtin(vertex_index)]] comes from.
            // render_pass.draw(0..self.num_vertices, 0..1);

            // When using an index buffer, you need to use draw_indexed.
            // The draw method ignores the index buffer.
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn main() {
    // Logging is Important
    //
    // wgpu panics with generic error messages
    // that aren't helpful. The good stuff is
    // logged just before panic.
    // env_logger::init();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // State::new uses async code, so we're going to wait for it to finish
    let mut state = pollster::block_on(State::new(&window));

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::RedrawRequested(_) => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &&mut so we have to dereference it twice
                            state.resize(**new_inner_size);
                        }
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        _ => {}
                    };
                }
            }
            _ => {}
        }
    });
}
