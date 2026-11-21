//! The cube of the WebGL tab, drawn with WebGPU, a picture from a file, and numbers doubled by
//! the GPU.
//!
//! The cube's vertices, their colors and its indices go to the GPU once, into buffers, and a
//! pipeline of WGSL shaders draws them. Then, before every frame the browser draws, the cube's turn
//! is worked out in Rust, written into a uniform buffer, and drawn by a render pass with 4 samples
//! a pixel, so its edges are smooth, and a depth texture as big as the canvas.
//!
//! The picture is an image the browser loads into a texture, with smaller images of it the GPU
//! makes, drawn on half the pixels of its canvas, through a layout of its own. Once it is drawn, its
//! middle pixel is read back. The numbers are doubled by a compute pass and read back too.
//!
//! Where the browser has no WebGPU, the tab says so. When the GPU device is lost, the browser
//! asks for another, and the cube's objects are made again on it.

use std::{cell::RefCell, f32::consts::PI, rc::Rc};

use vertigo::{
    DomNode, Value, component, css, dom, dom_element, get_driver, include_static,
    webgpu::{
        AddressMode, BindGroup, BindGroupEntry, BindGroupLayoutEntry, BindingResource, BindingType,
        Buffer, BufferUsages, Color, ColorAttachment, ColorTargetState, CompareFunction,
        ComputePipelineDescriptor, DepthAttachment, DepthStencilState, Face, FilterMode,
        FragmentState, ImageLoad, ImageOptions, IndexFormat, LoadOp, MultisampleState,
        PrimitiveState, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
        SamplerBindingType, SamplerDescriptor, ShaderStages, Target, TextureFormat,
        TextureSampleType, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState,
        VertexStepMode, WebGpu, WebGpuOptions,
    },
};

use super::webgl::{
    FACES, FrameCounter, Matrix, css_canvas, css_fps, css_frame, css_wrapper, multiply,
    perspective, rotation_x, rotation_y, translation,
};

#[component]
pub fn WebGpuDemo() {
    let canvas = dom_element! { <canvas css={css_canvas()} /> };
    let gpu = WebGpu::new(&canvas.get_ref());
    let cube = Rc::new(RefCell::new(Cube::new(&gpu)));

    canvas.append_drop_resource(gpu.on_restore({
        let (gpu, cube) = (gpu.clone(), cube.clone());
        move || *cube.borrow_mut() = Cube::new(&gpu)
    }));

    let unavailable = Value::new(false);
    canvas.append_drop_resource(gpu.on_unavailable({
        let unavailable = unavailable.clone();
        move || unavailable.set(true)
    }));

    let fps = Value::new(None);
    let counter = FrameCounter::default();
    canvas.append_drop_resource(get_driver().on_animation_frame({
        let fps = fps.clone();
        move |time| {
            cube.borrow().draw(time);
            if let Some(count) = counter.frame(time) {
                fps.set(Some(count));
            }
        }
    }));
    let fps = fps.map(|fps| match fps {
        Some(fps) => format!("{fps} FPS"),
        None => "… FPS".to_string(),
    });
    let note = unavailable.render_value_option(|unavailable| {
        unavailable.then(|| {
            dom! { <p>"This browser has no WebGPU, or no GPU for it: the canvas stays empty."</p> }
        })
    });

    dom! {
        <div css={css_wrapper()}>
            <p>"The cube of the WebGL tab, drawn with WebGPU, its turn worked out in Rust for every frame, its edges smooth."</p>
            {note}
            <div css={css_frame()}>
                {canvas}
                <span css={css_fps()}>{fps}</span>
            </div>
            {picture()}
            {doubled()}
        </div>
    }
}

const PICTURE: &str = "
struct Corner {
    @builtin(position) position: vec4f,
    @location(0) at: vec2f,
};

@group(0) @binding(0) var image: texture_2d<f32>;
@group(0) @binding(1) var between: sampler;

// Two triangles over the canvas, the image's top left at its top left.
@vertex fn vertex(@builtin(vertex_index) index: u32) -> Corner {
    let corners = array(vec2f(0.0, 0.0), vec2f(1.0, 0.0), vec2f(0.0, 1.0),
        vec2f(0.0, 1.0), vec2f(1.0, 0.0), vec2f(1.0, 1.0));
    let at = corners[index];
    return Corner(vec4f(at.x * 2.0 - 1.0, 1.0 - at.y * 2.0, 0.0, 1.0), at);
}

@fragment fn fragment(corner: Corner) -> @location(0) vec4f {
    return textureSample(image, between, corner.at);
}
";

/// An image the browser loads, drawn on half the pixels of its canvas, with a layout of its own,
/// and its middle pixel read back once it is drawn.
fn picture() -> DomNode {
    let canvas = dom_element! { <canvas css={css_picture()} /> };
    let gpu = WebGpu::with_options(
        &canvas.get_ref(),
        WebGpuOptions {
            bitmap_scale: 0.5,
            ..WebGpuOptions::default()
        },
    );
    let image = gpu.create_texture_from_url(
        &include_static!("../styling/counter.webp"),
        &ImageOptions {
            mipmaps: true,
            ..ImageOptions::default()
        },
    );
    let sampler = gpu.create_sampler(&SamplerDescriptor {
        address_mode_u: AddressMode::ClampToEdge,
        address_mode_v: AddressMode::ClampToEdge,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        mipmap_filter: FilterMode::Linear,
    });
    let layout = gpu.create_bind_group_layout(&[
        BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Texture {
                sample_type: TextureSampleType::Float { filterable: true },
                multisampled: false,
            },
        },
        BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Sampler(SamplerBindingType::Filtering),
        },
    ]);
    let shader = gpu.create_shader_module(PICTURE);
    let pipeline = gpu.create_render_pipeline(&RenderPipelineDescriptor {
        layout: Some(&gpu.create_pipeline_layout(&[&layout])),
        ..RenderPipelineDescriptor::new(
            VertexState {
                module: &shader,
                entry_point: "vertex",
                buffers: &[],
            },
            Some(FragmentState {
                module: &shader,
                entry_point: "fragment",
                targets: &[ColorTargetState::canvas()],
            }),
        )
    });
    // Takes the image when it comes.
    let group = gpu.create_bind_group(
        &layout,
        &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::Texture(&image),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(&sampler),
            },
        ],
    );

    let middle = Value::new("The middle of the picture: …".to_string());
    // The handles let their objects go when dropped: kept for as long as the canvas is.
    let kept = (image, sampler);
    let draw = Rc::new({
        let (gpu, middle) = (gpu.clone(), middle.clone());
        move |read: bool| {
            let _ = &kept;
            let mut pass = gpu.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[ColorAttachment::canvas(LoadOp::Clear(Color::WHITE))],
                depth_stencil_attachment: None,
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &group);
            pass.draw(0..6, 0..1);
            pass.end();
            if read {
                let middle = middle.clone();
                gpu.read_pixels(Target::Canvas, move |size, pixels| {
                    let at = ((size.height / 2 * size.width + size.width / 2) * 4) as usize;
                    if let Some([red, green, blue, _]) = pixels.get(at..at + 4) {
                        middle.set(format!(
                            "The middle of the picture: rgb({red}, {green}, {blue}), of {} x {} pixels",
                            size.width, size.height
                        ));
                    }
                });
            }
        }
    });
    canvas.append_drop_resource(gpu.on_resize({
        let draw = draw.clone();
        move |_| draw(false)
    }));
    canvas.append_drop_resource(gpu.on_image_load(move |load| {
        if let ImageLoad::Loaded { .. } = load {
            draw(true);
        }
    }));

    dom! {
        <div>
            <p>"A picture from a file, its smaller images made by the GPU, on half the pixels:"</p>
            {canvas}
            <p>{middle}</p>
        </div>
    }
}

const DOUBLING: &str = "
@group(0) @binding(0) var<storage, read_write> numbers: array<f32>;

@compute @workgroup_size(4) fn double(@builtin(global_invocation_id) id: vec3u) {
    numbers[id.x] *= 2.0;
}
";

/// 1, 2, 3 and 4, doubled by a compute pass, and read back.
fn doubled() -> DomNode {
    let canvas = dom_element! { <canvas style="display: none" /> };
    let gpu = WebGpu::new(&canvas.get_ref());
    let numbers = gpu.create_buffer_init(
        BufferUsages::STORAGE | BufferUsages::COPY_SRC,
        &[1.0_f32, 2.0, 3.0, 4.0],
    );
    let shader = gpu.create_shader_module(DOUBLING);
    let doubling = gpu.create_compute_pipeline(&ComputePipelineDescriptor {
        layout: None,
        module: &shader,
        entry_point: "double",
    });
    let group = gpu.create_bind_group(
        &doubling.get_bind_group_layout(0),
        &[BindGroupEntry {
            binding: 0,
            resource: BindingResource::Buffer(&numbers),
        }],
    );
    {
        let mut pass = gpu.begin_compute_pass();
        pass.set_pipeline(&doubling);
        pass.set_bind_group(0, &group);
        pass.dispatch_workgroups(1, 1, 1);
    }

    let text = Value::new("Doubled on the GPU: …".to_string());
    gpu.read_buffer(&numbers, {
        let text = text.clone();
        move |bytes| {
            let numbers: Vec<String> = bytes
                .as_chunks::<4>()
                .0
                .iter()
                .map(|bytes| f32::from_le_bytes(*bytes).to_string())
                .collect();
            text.set(format!("Doubled on the GPU: {}", numbers.join(", ")));
        }
    });
    // The context and the objects, kept for as long as the canvas is: their calls go nowhere once
    // they are dropped.
    let kept = (gpu.clone(), numbers, shader, doubling, group);
    canvas.append_drop_resource(gpu.on_restore(move || {
        let _ = &kept;
    }));

    dom! {
        <div>
            {canvas}
            <p>{text}</p>
        </div>
    }
}

fn css_picture() -> vertigo::Css {
    css! {"
        display: block;
        width: 200px;
        height: 208px;
        border: 1px solid #ccc;
    "}
}

const SHADER: &str = "
struct Corner {
    @builtin(position) position: vec4f,
    @location(0) color: vec3f,
};

@group(0) @binding(0) var<uniform> transform: mat4x4f;

@vertex fn vertex(@location(0) position: vec3f, @location(1) color: vec3f) -> Corner {
    return Corner(transform * vec4f(position, 1.0), color);
}

@fragment fn fragment(corner: Corner) -> @location(0) vec4f {
    return vec4f(corner.color, 1.0);
}
";

const DEPTH: TextureFormat = TextureFormat::Depth24Plus;

/// Of a pixel of the cube, for smooth edges.
const SAMPLES: u32 = 4;

/// From the depths of OpenGL, -1 to 1, which `perspective` gives, to those of WebGPU, 0 to 1.
#[rustfmt::skip]
const DEPTH_ZERO_TO_ONE: Matrix = [
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
];

struct Cube {
    gpu: WebGpu,
    pipeline: RenderPipeline,
    corners: Buffer,
    indices: Buffer,
    /// The turn, a matrix.
    transform: Buffer,
    group: BindGroup,
}

impl Cube {
    /// The cube's objects on `gpu`, made anew when the context has a new device.
    fn new(gpu: &WebGpu) -> Self {
        let gpu = gpu.clone();
        let shader = gpu.create_shader_module(SHADER);
        let pipeline = gpu.create_render_pipeline(&RenderPipelineDescriptor {
            vertex: VertexState {
                module: &shader,
                entry_point: "vertex",
                buffers: &[VertexBufferLayout {
                    array_stride: 24,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &[
                        VertexAttribute {
                            format: VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x3,
                            offset: 12,
                            shader_location: 1,
                        },
                    ],
                }],
            },
            primitive: PrimitiveState {
                cull_mode: Some(Face::Back),
                ..PrimitiveState::default()
            },
            depth_stencil: Some(DepthStencilState {
                format: DEPTH,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
            }),
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fragment",
                targets: &[ColorTargetState::canvas()],
            }),
            layout: None,
            multisample: MultisampleState { count: SAMPLES },
        });

        // Four corners a face, each a position and a color, and two triangles of them.
        let mut corners = Vec::new();
        let mut indices: Vec<u16> = Vec::new();
        for (face, (corner, across, up, color)) in FACES.iter().enumerate() {
            for (a, b) in [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)] {
                corners.extend((0..3).map(|axis| corner[axis] + a * across[axis] + b * up[axis]));
                corners.extend(color);
            }
            let first = face as u16 * 4;
            indices.extend([first, first + 1, first + 2, first, first + 2, first + 3]);
        }

        let corners = gpu.create_buffer_init(BufferUsages::VERTEX, &corners);
        let indices = gpu.create_buffer_init(BufferUsages::INDEX, &indices);
        let transform = gpu.create_buffer(64, BufferUsages::UNIFORM | BufferUsages::COPY_DST);
        let group = gpu.create_bind_group(
            &pipeline.get_bind_group_layout(0),
            &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(&transform),
            }],
        );

        Self {
            gpu,
            pipeline,
            corners,
            indices,
            transform,
            group,
        }
    }

    /// The cube at `time`, in milliseconds, turning as the one of the WebGL tab does.
    fn draw(&self, time: f64) {
        let seconds = (time / 1000.0) as f32;
        let aspect = self.gpu.size().aspect();
        let transform = multiply(
            &multiply(
                &multiply(
                    &DEPTH_ZERO_TO_ONE,
                    &perspective(PI / 4.0, aspect, 0.1, 100.0),
                ),
                &translation(0.0, 0.0, -6.0),
            ),
            &multiply(
                &rotation_x(seconds * PI / 5.0),
                &rotation_y(seconds * PI / 3.0),
            ),
        );
        self.gpu.write_buffer(&self.transform, 0, &transform);

        let mut pass = self.gpu.begin_render_pass(&RenderPassDescriptor {
            color_attachments: &[ColorAttachment::canvas_multisampled(
                SAMPLES,
                LoadOp::Clear(Color {
                    r: 0.95,
                    g: 0.95,
                    b: 0.97,
                    a: 1.0,
                }),
            )],
            depth_stencil_attachment: Some(DepthAttachment::canvas(DEPTH, LoadOp::Clear(1.0))),
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.group);
        pass.set_vertex_buffer(0, &self.corners, 0);
        pass.set_index_buffer(&self.indices, IndexFormat::Uint16, 0);
        pass.draw_indexed(0..36, 0, 0..1);
    }
}
