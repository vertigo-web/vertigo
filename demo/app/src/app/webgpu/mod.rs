//! The cube of the WebGL tab, drawn with WebGPU.
//!
//! Its vertices, their colors and its indices go to the GPU once, into buffers, and a pipeline
//! of WGSL shaders draws them. Then, before every frame the browser draws, the cube's turn is
//! worked out in Rust, written into a uniform buffer, and drawn by a render pass with a depth
//! texture as big as the canvas.
//!
//! Where the browser has no WebGPU, the tab says so. When the GPU device is lost, the browser
//! asks for another, and the cube's objects are made again on it.

use std::{cell::RefCell, f32::consts::PI, rc::Rc};

use vertigo::{
    Value, component, dom, dom_element, get_driver,
    webgpu::{
        BindGroup, BindGroupEntry, BindingResource, Buffer, BufferUsages, Color, ColorAttachment,
        ColorTargetState, CompareFunction, DepthAttachment, DepthStencilState, Face, FragmentState,
        IndexFormat, LoadOp, PrimitiveState, RenderPassDescriptor, RenderPipeline,
        RenderPipelineDescriptor, TextureFormat, VertexAttribute, VertexBufferLayout, VertexFormat,
        VertexState, VertexStepMode, WebGpu,
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
            <p>"The cube of the WebGL tab, drawn with WebGPU, its turn worked out in Rust for every frame."</p>
            {note}
            <div css={css_frame()}>
                {canvas}
                <span css={css_fps()}>{fps}</span>
            </div>
        </div>
    }
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
            &pipeline,
            0,
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
            color_attachments: &[ColorAttachment::canvas(LoadOp::Clear(Color {
                r: 0.95,
                g: 0.95,
                b: 0.97,
                a: 1.0,
            }))],
            depth_stencil_attachment: Some(DepthAttachment::canvas(DEPTH, LoadOp::Clear(1.0))),
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.group);
        pass.set_vertex_buffer(0, &self.corners, 0);
        pass.set_index_buffer(&self.indices, IndexFormat::Uint16, 0);
        pass.draw_indexed(0..36, 0, 0..1);
    }
}
