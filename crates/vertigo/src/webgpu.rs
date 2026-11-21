//! WebGPU on a `<canvas>`, see [`WebGpu`].
//!
//! ## Format
//!
//! The calls go to the browser as commands, which it reads in
//! `driver_module/src_js/api/command/webgpu.ts`, and `webgpu.test.ts` there checks it against
//! bytes made here. Numbers are little-endian.
//!
//! ```text
//! commands  := op*
//! op        := code:u8 args               -- see `code`
//! callback  := u64                        -- what the browser tells the context through, see
//!                                            `GpuEvent`
//! id        := u32                        -- of an object, 0 for none
//! source    := length:u32 utf8*           -- WGSL, or the name of an entry point
//! data      := length:u32 byte*
//! enum      := u8                         -- the place of a value in the list of WebGPU's
//!                                            strings for it, see the enums here
//! pipeline  := vertex primitive depth fragment
//! vertex    := module:id entry:source count:u8 (stride:u32 step:enum count:u8
//!              (format:enum offset:u32 location:u32)*)*
//! primitive := topology:enum strip_index_format:enum front_face:enum cull_mode:enum
//! depth     := 0 | 1 format:enum write:u8 compare:enum
//! fragment  := 0 | 1 module:id entry:source count:u8 (format:enum blend write_mask:u8)*
//! blend     := 0 | 1 (src:enum dst:enum operation:enum){2}  -- of color, then of alpha
//! entry     := binding:u32 (0 buffer:id offset:u32 size:u32 | 1 texture:id | 2 sampler:id)
//! pass      := count:u8 (target load:(0 | 1 r:f64 g:f64 b:f64 a:f64) store:u8)*
//!              (0 | 1 depth_target load:(0 | 1 depth:f32) store:u8)
//! target    := 0 | 1 texture:id           -- the canvas, or a texture
//! depth_target := 0 format:enum | 1 texture:id  -- one as big as the canvas, or a texture
//! ```
//!
//! A WebGPU flag, f. ex. of [`BufferUsages`], travels as its number, a `u32`, and a size of
//! 0 for a buffer's binding means the rest of it.

use std::{
    cell::Cell,
    fmt,
    ops::{BitOr, Range},
    rc::{Rc, Weak},
};

use crate::{
    DomElementRef, DomId, DropResource, JsJson,
    dev::{JsJsonListDecoder, command::DriverDomCommand},
    driver_module::{
        CanvasContext, api::api_callbacks, event_emitter::EventEmitter, get_driver_dom,
    },
    struct_mut::ValueMut,
};

pub use crate::webgl::{Scalar, Size};

/// Operation codes. Append only, as the browser reads them by number.
mod code {
    /// The context's options, see `WebGpuOptions`, and its callback: always the first command
    /// of a `WebGpu`.
    pub const CONTEXT: u8 = 1;
    pub const CREATE_BUFFER: u8 = 2;
    pub const DESTROY_BUFFER: u8 = 3;
    pub const WRITE_BUFFER: u8 = 4;
    pub const CREATE_TEXTURE: u8 = 5;
    pub const DESTROY_TEXTURE: u8 = 6;
    pub const WRITE_TEXTURE: u8 = 7;
    pub const CREATE_SAMPLER: u8 = 8;
    pub const DELETE_SAMPLER: u8 = 9;
    pub const CREATE_SHADER_MODULE: u8 = 10;
    pub const DELETE_SHADER_MODULE: u8 = 11;
    pub const CREATE_RENDER_PIPELINE: u8 = 12;
    pub const CREATE_COMPUTE_PIPELINE: u8 = 13;
    pub const DELETE_PIPELINE: u8 = 14;
    pub const CREATE_BIND_GROUP: u8 = 15;
    pub const DELETE_BIND_GROUP: u8 = 16;
    pub const BEGIN_RENDER_PASS: u8 = 17;
    pub const BEGIN_COMPUTE_PASS: u8 = 18;
    pub const END_PASS: u8 = 19;
    pub const SET_PIPELINE: u8 = 20;
    pub const SET_BIND_GROUP: u8 = 21;
    pub const SET_VERTEX_BUFFER: u8 = 22;
    pub const SET_INDEX_BUFFER: u8 = 23;
    pub const DRAW: u8 = 24;
    pub const DRAW_INDEXED: u8 = 25;
    pub const DISPATCH_WORKGROUPS: u8 = 26;
    pub const SET_VIEWPORT: u8 = 27;
    pub const SET_SCISSOR_RECT: u8 = 28;
}

/// What the browser tells of a context, as `webgpu.ts` sends it: `[0, width, height]` when the
/// canvas changes its size, `[1]` when the context has a new device after it lost its own, `[2]`
/// when the browser has no WebGPU for it.
enum GpuEvent {
    Resized(Size),
    Restored,
    Unavailable,
}

impl GpuEvent {
    fn from_driver(data: JsJson) -> Result<Self, String> {
        data.map_list(|mut params: JsJsonListDecoder| {
            let event = match params.get_u32("kind")? {
                0 => Self::Resized(Size {
                    width: params.get_u32("width")?,
                    height: params.get_u32("height")?,
                }),
                1 => Self::Restored,
                2 => Self::Unavailable,
                other => return Err(format!("unknown gpu event {other}")),
            };
            params.expect_no_more()?;
            Ok(event)
        })
    }
}

/// A context: its calls not yet sent, shared by it and its objects, which the browser gets at the
/// end of the transaction (see `DriverDom::flush_dom_changes`), and what the browser told of it.
struct Context {
    queue: ValueMut<Vec<u8>>,
    size: Cell<Size>,
    resized: EventEmitter<Size>,
    restored: EventEmitter<()>,
    unavailable: Cell<bool>,
    told_unavailable: EventEmitter<()>,
    /// The callback the browser tells through, for as long as the context is.
    _callback: DropResource,
}

impl Context {
    fn new(options: WebGpuOptions) -> Rc<Self> {
        Rc::new_cyclic(|context: &Weak<Self>| {
            let (callback, drop) = api_callbacks().register({
                let context = context.clone();
                move |data| {
                    if let Some(context) = context.upgrade() {
                        context.told(data);
                    }
                    JsJson::Null
                }
            });

            let mut queue = vec![
                code::CONTEXT,
                options.alpha_mode as u8,
                options.power_preference as u8,
            ];
            queue.extend_from_slice(&callback.as_u64().to_le_bytes());

            Self {
                queue: ValueMut::new(queue),
                size: Cell::new(Size::default()),
                resized: EventEmitter::default(),
                restored: EventEmitter::default(),
                unavailable: Cell::new(false),
                told_unavailable: EventEmitter::default(),
                _callback: drop,
            }
        })
    }

    fn told(&self, data: JsJson) {
        match GpuEvent::from_driver(data) {
            Ok(GpuEvent::Resized(size)) => {
                self.size.set(size);
                self.resized.trigger(&size);
            }
            Ok(GpuEvent::Restored) => self.restored.trigger(&()),
            Ok(GpuEvent::Unavailable) => {
                self.unavailable.set(true);
                self.told_unavailable.trigger(&());
            }
            Err(error) => log::error!("webgpu: {error}"),
        }
    }
}

impl CanvasContext for Context {
    fn take_command(&self, id: DomId) -> Option<DriverDomCommand> {
        let commands = self.queue.change(std::mem::take);
        (!commands.is_empty()).then_some(DriverDomCommand::WebGpu { id, commands })
    }
}

thread_local! {
    /// Ids of objects, unique among all contexts.
    static LAST_ID: Cell<u32> = const { Cell::new(0) };
}

/// How the canvas gets its WebGPU context. The first [`WebGpu`] of a canvas sets them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WebGpuOptions {
    pub alpha_mode: AlphaMode,
    /// Which GPU the browser is asked for, where there are more.
    pub power_preference: PowerPreference,
}

/// Whether the canvas is see-through where the picture is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum AlphaMode {
    /// Not: the alpha of the picture is left out.
    #[default]
    Opaque = 0,
    /// As much as the alpha of the picture, whose colors are multiplied by it.
    Premultiplied = 1,
}

/// Which GPU the browser is asked for, see [`WebGpuOptions`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum PowerPreference {
    /// As the browser chooses.
    #[default]
    Default = 0,
    /// The one that saves the battery, f. ex. the one in the processor.
    LowPower = 1,
    /// The fastest.
    HighPerformance = 2,
}

/// WebGPU on a `<canvas>`: its calls, with handles in place of WebGPU's objects, which are let
/// go when they are dropped. Its shapes follow those of [wgpu](https://docs.rs/wgpu), cut down
/// to what most pictures need.
///
/// The calls of a transaction go to the browser together at its end, in the same batch as the
/// changes to the page, after the canvas is on it. So a frame is drawn by calls made in
/// [`Driver::on_animation_frame`](crate::Driver::on_animation_frame): render passes, see
/// [`Self::begin_render_pass`], whose commands are sent to the GPU at the end of the batch, after
/// what [`Self::write_buffer`] and [`Self::write_texture`] wrote before them. The objects the
/// passes use are made once, beforehand. Nothing is drawn during SSR.
///
/// No call waits for the browser to answer. The browser asks for a GPU device first, and the
/// calls coming meanwhile wait for it. Where the browser has no WebGPU, or no GPU for it, the
/// calls go nowhere and [`Self::on_unavailable`] tells it: the canvas is still free for
/// [`WebGl`](crate::webgl::WebGl). A shader that doesn't compile, or any call WebGPU finds
/// wrong, is reported in the browser's console.
///
/// Bind groups take their layouts from the pipelines, as WebGPU's `layout: "auto"` makes them,
/// see [`Self::create_bind_group`].
///
/// The canvas's picture is as big as the canvas is on the screen, in device pixels, so it is
/// sharp, and so is a depth texture of the canvas, see [`DepthTarget::Canvas`].
///
/// ```rust
/// use vertigo::{DomNode, dom_element, get_driver};
/// use vertigo::webgpu::{
///     BindGroupEntry, BindingResource, BufferUsages, Color, ColorAttachment, ColorTargetState,
///     FragmentState, LoadOp, PrimitiveState, RenderPassDescriptor, RenderPipelineDescriptor,
///     VertexState, WebGpu,
/// };
///
/// // A triangle, its color going round.
/// fn triangle() -> DomNode {
///     let canvas = dom_element! { <canvas style="width: 200px; height: 200px" /> };
///     let gpu = WebGpu::new(&canvas.get_ref());
///
///     let shader = gpu.create_shader_module("
///         @group(0) @binding(0) var<uniform> color: vec4f;
///
///         @vertex fn vertex(@builtin(vertex_index) index: u32) -> @builtin(position) vec4f {
///             let corners = array(vec2f(0.0, 0.8), vec2f(-0.8, -0.8), vec2f(0.8, -0.8));
///             return vec4f(corners[index], 0.0, 1.0);
///         }
///
///         @fragment fn fragment() -> @location(0) vec4f {
///             return color;
///         }
///     ");
///     let pipeline = gpu.create_render_pipeline(&RenderPipelineDescriptor {
///         vertex: VertexState { module: &shader, entry_point: "vertex", buffers: &[] },
///         primitive: PrimitiveState::default(),
///         depth_stencil: None,
///         fragment: Some(FragmentState {
///             module: &shader,
///             entry_point: "fragment",
///             targets: &[ColorTargetState::canvas()],
///         }),
///     });
///     let color = gpu.create_buffer(16, BufferUsages::UNIFORM | BufferUsages::COPY_DST);
///     let colors = gpu.create_bind_group(&pipeline, 0, &[BindGroupEntry {
///         binding: 0,
///         resource: BindingResource::Buffer(&color),
///     }]);
///
///     canvas.append_drop_resource(get_driver().on_animation_frame(move |time| {
///         let turn = (time / 1000.0) as f32;
///         gpu.write_buffer(&color, 0, &[turn.sin().abs(), turn.cos().abs(), 0.5, 1.0]);
///
///         let mut pass = gpu.begin_render_pass(&RenderPassDescriptor {
///             color_attachments: &[ColorAttachment::canvas(LoadOp::Clear(Color::BLACK))],
///             depth_stencil_attachment: None,
///         });
///         pass.set_pipeline(&pipeline);
///         pass.set_bind_group(0, &colors);
///         pass.draw(0..3, 0..1);
///     }));
///
///     canvas.into()
/// }
/// ```
#[derive(Clone)]
pub struct WebGpu {
    context: Rc<Context>,
}

impl fmt::Debug for WebGpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebGpu").finish_non_exhaustive()
    }
}

impl WebGpu {
    /// WebGPU on `canvas`, opaque, on the GPU the browser chooses.
    pub fn new(canvas: &DomElementRef) -> Self {
        Self::with_options(canvas, WebGpuOptions::default())
    }

    /// WebGPU on `canvas`, with a context of these options.
    ///
    /// ```rust
    /// use vertigo::dom_element;
    /// use vertigo::webgpu::{AlphaMode, PowerPreference, WebGpu, WebGpuOptions};
    ///
    /// let canvas = dom_element! { <canvas /> };
    /// let gpu = WebGpu::with_options(&canvas.get_ref(), WebGpuOptions {
    ///     alpha_mode: AlphaMode::Premultiplied,
    ///     power_preference: PowerPreference::LowPower,
    /// });
    /// ```
    pub fn with_options(canvas: &DomElementRef, options: WebGpuOptions) -> Self {
        let context = Context::new(options);
        let weak: Weak<Context> = Rc::downgrade(&context);
        get_driver_dom().canvas_context(canvas.id(), weak);
        Self { context }
    }

    /// The size of the canvas's picture, in device pixels: as big as the canvas is on the
    /// screen, so the picture is sharp. 0 x 0 until the browser has measured the canvas, soon
    /// after the first calls, and during SSR.
    pub fn size(&self) -> Size {
        self.context.size.get()
    }

    /// `callback` with the canvas's new size whenever it changes, first once the browser has
    /// measured the canvas. The picture gets the size with the passes that come next: those
    /// `callback` makes, if it draws, so the canvas never shows it stretched. A scene drawn once
    /// is drawn again here, and one drawn every frame needs nothing, but its projection can take
    /// [`Size::aspect`].
    ///
    /// ```rust
    /// use vertigo::dom_element;
    /// use vertigo::webgpu::{Color, ColorAttachment, LoadOp, RenderPassDescriptor, WebGpu};
    ///
    /// let canvas = dom_element! { <canvas style="width: 100%; height: 50vh" /> };
    /// let gpu = WebGpu::new(&canvas.get_ref());
    ///
    /// // A picture that doesn't move: drawn now and after each resize.
    /// let draw = {
    ///     let gpu = gpu.clone();
    ///     move || {
    ///         gpu.begin_render_pass(&RenderPassDescriptor {
    ///             color_attachments: &[ColorAttachment::canvas(LoadOp::Clear(Color {
    ///                 r: 0.1, g: 0.1, b: 0.4, a: 1.0,
    ///             }))],
    ///             depth_stencil_attachment: None,
    ///         });
    ///     }
    /// };
    /// draw();
    /// canvas.append_drop_resource(gpu.on_resize(move |_size| draw()));
    /// ```
    pub fn on_resize(&self, callback: impl Fn(Size) + 'static) -> DropResource {
        self.context.resized.add(callback)
    }

    /// `callback` when the context has a new GPU device after it lost its own, f. ex. when the
    /// graphics driver restarted. Every object made before is gone then: `callback` makes the
    /// objects again and draws. Calls between the loss and this do nothing.
    pub fn on_restore(&self, callback: impl Fn() + 'static) -> DropResource {
        self.context.restored.add(move |()| callback())
    }

    /// `callback` once the browser finds it has no WebGPU, or no GPU for it, or it lost the
    /// device and got no other, at once if it found that already. The calls go nowhere then,
    /// and the canvas is free for [`WebGl`](crate::webgl::WebGl), if it hasn't the context
    /// already.
    ///
    /// ```rust
    /// use vertigo::{Value, dom_element};
    /// use vertigo::webgpu::WebGpu;
    ///
    /// let canvas = dom_element! { <canvas /> };
    /// let gpu = WebGpu::new(&canvas.get_ref());
    /// let fallback = Value::new(false);
    /// let unavailable = gpu.on_unavailable({
    ///     let fallback = fallback.clone();
    ///     move || fallback.set(true)
    /// });
    /// ```
    pub fn on_unavailable(&self, callback: impl Fn() + 'static) -> DropResource {
        if self.context.unavailable.get() {
            callback();
        }
        self.context.told_unavailable.add(move |()| callback())
    }

    fn op(&self, code: u8, write: impl FnOnce(&mut Vec<u8>)) {
        self.context.queue.change(|out| {
            out.push(code);
            write(out);
        });
    }

    /// An object made with `create`, its arguments written by `write` after its id, and let go
    /// with `delete` when its handle is dropped.
    fn object(&self, create: u8, delete: u8, write: impl FnOnce(&mut Vec<u8>)) -> Object {
        let id = LAST_ID.with(|last| {
            last.set(last.get() + 1);
            last.get()
        });
        self.op(create, |out| {
            put_u32(out, id);
            write(out);
        });
        Object {
            id,
            delete,
            context: Rc::downgrade(&self.context),
        }
    }

    /// A buffer of `size` bytes, rounded up to a multiple of 4, as [`Self::write_buffer`] writes
    /// such, and zeroed.
    pub fn create_buffer(&self, size: u32, usage: BufferUsages) -> Buffer {
        let size = size.next_multiple_of(4);
        Buffer {
            object: self.object(code::CREATE_BUFFER, code::DESTROY_BUFFER, |out| {
                put_u32(out, size);
                put_u32(out, usage.0);
            }),
            size,
        }
    }

    /// A buffer with `data` in it, which can be written again.
    ///
    /// ```rust
    /// use vertigo::dom_element;
    /// use vertigo::webgpu::{BufferUsages, WebGpu};
    ///
    /// let canvas = dom_element! { <canvas /> };
    /// let gpu = WebGpu::new(&canvas.get_ref());
    /// // Three indices of 2 bytes, in a buffer of 8.
    /// let indices = gpu.create_buffer_init(BufferUsages::INDEX, &[0_u16, 1, 2]);
    /// assert_eq!(indices.size(), 8);
    /// ```
    pub fn create_buffer_init<T: Scalar>(&self, usage: BufferUsages, data: &[T]) -> Buffer {
        let buffer = self.create_buffer(
            std::mem::size_of_val(data) as u32,
            usage | BufferUsages::COPY_DST,
        );
        self.write_buffer(&buffer, 0, data);
        buffer
    }

    /// Writes `data` into `buffer`, from `offset` bytes, a multiple of 4, before the passes that
    /// come after. Bytes of 0 make it a multiple of 4 bytes long, as WebGPU wants it.
    pub fn write_buffer<T: Scalar>(&self, buffer: &Buffer, offset: u32, data: &[T]) {
        self.op(code::WRITE_BUFFER, |out| {
            put_u32(out, buffer.object.id);
            put_u32(out, offset);
            let size = std::mem::size_of_val(data);
            put_u32(out, size.next_multiple_of(4) as u32);
            for value in data {
                value.put(out);
            }
            out.resize(out.len() + size.next_multiple_of(4) - size, 0);
        });
    }

    /// A 2D texture, of one level, see [`Self::write_texture`].
    pub fn create_texture(&self, descriptor: &TextureDescriptor) -> Texture {
        let TextureDescriptor {
            size,
            format,
            usage,
        } = *descriptor;
        Texture {
            object: self.object(code::CREATE_TEXTURE, code::DESTROY_TEXTURE, |out| {
                put_u32(out, size.width);
                put_u32(out, size.height);
                out.push(format as u8);
                put_u32(out, usage.0);
            }),
            size,
            format,
        }
    }

    /// Writes all of `texture`'s pixels, row after row from the top, each as its format has it,
    /// f. ex. 4 bytes of red, green, blue and alpha. The texture needs
    /// [`TextureUsages::COPY_DST`].
    ///
    /// ```rust
    /// use vertigo::dom_element;
    /// use vertigo::webgpu::{Size, TextureDescriptor, TextureFormat, TextureUsages, WebGpu};
    ///
    /// let canvas = dom_element! { <canvas /> };
    /// let gpu = WebGpu::new(&canvas.get_ref());
    /// // Black and white, two by two.
    /// let checker = gpu.create_texture(&TextureDescriptor {
    ///     size: Size { width: 2, height: 2 },
    ///     format: TextureFormat::Rgba8Unorm,
    ///     usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
    /// });
    /// gpu.write_texture(&checker, &[
    ///     0, 0, 0, 255, 255, 255, 255, 255,
    ///     255, 255, 255, 255, 0, 0, 0, 255,
    /// ]);
    /// ```
    pub fn write_texture(&self, texture: &Texture, data: &[u8]) {
        self.op(code::WRITE_TEXTURE, |out| {
            put_u32(out, texture.object.id);
            put_u32(out, texture.size.width * texture.format.bytes_per_pixel());
            put_u32(out, texture.size.width);
            put_u32(out, texture.size.height);
            put_data(out, data);
        });
    }

    /// How shaders read a texture between its pixels and outside of it.
    pub fn create_sampler(&self, descriptor: &SamplerDescriptor) -> Sampler {
        let SamplerDescriptor {
            address_mode_u,
            address_mode_v,
            mag_filter,
            min_filter,
            mipmap_filter,
        } = *descriptor;
        Sampler(
            self.object(code::CREATE_SAMPLER, code::DELETE_SAMPLER, |out| {
                out.extend([
                    address_mode_u as u8,
                    address_mode_v as u8,
                    mag_filter as u8,
                    min_filter as u8,
                    mipmap_filter as u8,
                ]);
            }),
        )
    }

    /// Shaders in [WGSL](https://www.w3.org/TR/WGSL/), whose entry points pipelines take.
    pub fn create_shader_module(&self, wgsl: &str) -> ShaderModule {
        ShaderModule(self.object(
            code::CREATE_SHADER_MODULE,
            code::DELETE_SHADER_MODULE,
            |out| put_source(out, wgsl),
        ))
    }

    /// How a render pass draws, see [`RenderPipelineDescriptor`].
    pub fn create_render_pipeline(
        &self,
        descriptor: &RenderPipelineDescriptor<'_>,
    ) -> RenderPipeline {
        RenderPipeline(
            self.object(code::CREATE_RENDER_PIPELINE, code::DELETE_PIPELINE, |out| {
                descriptor.put(out)
            }),
        )
    }

    /// How a compute pass computes: the entry point `entry_point` of `module`.
    pub fn create_compute_pipeline(
        &self,
        module: &ShaderModule,
        entry_point: &str,
    ) -> ComputePipeline {
        ComputePipeline(self.object(
            code::CREATE_COMPUTE_PIPELINE,
            code::DELETE_PIPELINE,
            |out| {
                put_u32(out, module.0.id);
                put_source(out, entry_point);
            },
        ))
    }

    /// The resources of the bind group `group` of `pipeline`'s shaders, by their bindings:
    /// `@group(group) @binding(binding)`. The bind group is for `pipeline` only, as its layout is
    /// the one WebGPU found in the shaders of it.
    pub fn create_bind_group(
        &self,
        pipeline: &impl Pipeline,
        group: u32,
        entries: &[BindGroupEntry<'_>],
    ) -> BindGroup {
        BindGroup(
            self.object(code::CREATE_BIND_GROUP, code::DELETE_BIND_GROUP, |out| {
                put_u32(out, pipeline.object().id);
                put_u32(out, group);
                out.push(entries.len() as u8);
                for BindGroupEntry { binding, resource } in entries {
                    put_u32(out, *binding);
                    match resource {
                        BindingResource::Buffer(buffer) => {
                            out.push(0);
                            put_u32(out, buffer.object.id);
                            put_u32(out, 0);
                            put_u32(out, 0);
                        }
                        BindingResource::BufferRange {
                            buffer,
                            offset,
                            size,
                        } => {
                            out.push(0);
                            put_u32(out, buffer.object.id);
                            put_u32(out, *offset);
                            put_u32(out, *size);
                        }
                        BindingResource::Texture(texture) => {
                            out.push(1);
                            put_u32(out, texture.object.id);
                        }
                        BindingResource::Sampler(sampler) => {
                            out.push(2);
                            put_u32(out, sampler.0.id);
                        }
                    }
                }
            }),
        )
    }

    /// A render pass, drawing into the attachments of `descriptor`, ended when it is dropped.
    pub fn begin_render_pass(&self, descriptor: &RenderPassDescriptor<'_>) -> RenderPass<'_> {
        self.op(code::BEGIN_RENDER_PASS, |out| descriptor.put(out));
        RenderPass { gpu: self }
    }

    /// A compute pass, ended when it is dropped.
    pub fn begin_compute_pass(&self) -> ComputePass<'_> {
        self.op(code::BEGIN_COMPUTE_PASS, |_| {});
        ComputePass { gpu: self }
    }
}

/// A WebGPU object, let go when dropped. Its context may be gone by then, and with it the object.
struct Object {
    id: u32,
    delete: u8,
    context: Weak<Context>,
}

impl Drop for Object {
    fn drop(&mut self) {
        if let Some(context) = self.context.upgrade() {
            context.queue.change(|out| {
                out.push(self.delete);
                put_u32(out, self.id);
            });
        }
    }
}

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Object").field(&self.id).finish()
    }
}

/// Memory of the GPU, see [`WebGpu::create_buffer`].
#[derive(Debug)]
pub struct Buffer {
    object: Object,
    size: u32,
}

impl Buffer {
    /// In bytes.
    pub fn size(&self) -> u32 {
        self.size
    }
}

/// A 2D texture, see [`WebGpu::create_texture`].
#[derive(Debug)]
pub struct Texture {
    object: Object,
    size: Size,
    format: TextureFormat,
}

impl Texture {
    pub fn size(&self) -> Size {
        self.size
    }

    pub fn format(&self) -> TextureFormat {
        self.format
    }
}

/// See [`WebGpu::create_sampler`].
#[derive(Debug)]
pub struct Sampler(Object);

/// See [`WebGpu::create_shader_module`].
#[derive(Debug)]
pub struct ShaderModule(Object);

/// See [`WebGpu::create_render_pipeline`].
#[derive(Debug)]
pub struct RenderPipeline(Object);

/// See [`WebGpu::create_compute_pipeline`].
#[derive(Debug)]
pub struct ComputePipeline(Object);

/// See [`WebGpu::create_bind_group`].
#[derive(Debug)]
pub struct BindGroup(Object);

mod sealed {
    pub trait Sealed {}
}

/// A render or a compute pipeline, whose bind groups [`WebGpu::create_bind_group`] makes.
pub trait Pipeline: sealed::Sealed {
    #[doc(hidden)]
    #[allow(private_interfaces)]
    fn object(&self) -> &Object;
}

impl sealed::Sealed for RenderPipeline {}

impl Pipeline for RenderPipeline {
    #[allow(private_interfaces)]
    fn object(&self) -> &Object {
        &self.0
    }
}

impl sealed::Sealed for ComputePipeline {}

impl Pipeline for ComputePipeline {
    #[allow(private_interfaces)]
    fn object(&self) -> &Object {
        &self.0
    }
}

/// Commands drawing, see [`WebGpu::begin_render_pass`]. The pass ends when it is dropped.
pub struct RenderPass<'a> {
    gpu: &'a WebGpu,
}

impl RenderPass<'_> {
    pub fn set_pipeline(&mut self, pipeline: &RenderPipeline) {
        self.gpu
            .op(code::SET_PIPELINE, |out| put_u32(out, pipeline.0.id));
    }

    /// The bind group of the shaders' `@group(index)`.
    pub fn set_bind_group(&mut self, index: u32, group: &BindGroup) {
        set_bind_group(self.gpu, index, group);
    }

    /// The vertex buffer `slot`, as the pipeline's `buffers` have them, from `offset` bytes.
    pub fn set_vertex_buffer(&mut self, slot: u32, buffer: &Buffer, offset: u32) {
        self.gpu.op(code::SET_VERTEX_BUFFER, |out| {
            put_u32(out, slot);
            put_u32(out, buffer.object.id);
            put_u32(out, offset);
        });
    }

    /// The indices [`Self::draw_indexed`] takes, from `offset` bytes.
    pub fn set_index_buffer(&mut self, buffer: &Buffer, format: IndexFormat, offset: u32) {
        self.gpu.op(code::SET_INDEX_BUFFER, |out| {
            put_u32(out, buffer.object.id);
            out.push(format as u8);
            put_u32(out, offset);
        });
    }

    /// The `vertices` of the `instances`.
    pub fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>) {
        self.gpu.op(code::DRAW, |out| {
            put_u32(out, vertices.len() as u32);
            put_u32(out, instances.len() as u32);
            put_u32(out, vertices.start);
            put_u32(out, instances.start);
        });
    }

    /// The vertices of the `indices` of the index buffer, each added to `base_vertex`, of the
    /// `instances`.
    pub fn draw_indexed(&mut self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>) {
        self.gpu.op(code::DRAW_INDEXED, |out| {
            put_u32(out, indices.len() as u32);
            put_u32(out, instances.len() as u32);
            put_u32(out, indices.start);
            out.extend_from_slice(&base_vertex.to_le_bytes());
            put_u32(out, instances.start);
        });
    }

    /// Where on the attachments the pass draws, in pixels from the top left corner, and the
    /// depths it draws at. All of them until this.
    pub fn set_viewport(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        min_depth: f32,
        max_depth: f32,
    ) {
        self.gpu.op(code::SET_VIEWPORT, |out| {
            put_f32s(out, &[x, y, width, height, min_depth, max_depth]);
        });
    }

    /// Where on the attachments the pass may change pixels. All of them until this.
    pub fn set_scissor_rect(&mut self, x: u32, y: u32, width: u32, height: u32) {
        self.gpu.op(code::SET_SCISSOR_RECT, |out| {
            for value in [x, y, width, height] {
                put_u32(out, value);
            }
        });
    }

    /// Ends the pass, as dropping it does.
    pub fn end(self) {}
}

impl Drop for RenderPass<'_> {
    fn drop(&mut self) {
        self.gpu.op(code::END_PASS, |_| {});
    }
}

impl fmt::Debug for RenderPass<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RenderPass").finish_non_exhaustive()
    }
}

/// Commands computing, see [`WebGpu::begin_compute_pass`]. The pass ends when it is dropped.
///
/// ```rust
/// use vertigo::dom_element;
/// use vertigo::webgpu::{BindGroupEntry, BindingResource, BufferUsages, WebGpu};
///
/// let canvas = dom_element! { <canvas /> };
/// let gpu = WebGpu::new(&canvas.get_ref());
/// // Each of 256 numbers doubled.
/// let numbers = gpu.create_buffer_init(BufferUsages::STORAGE, &[1.0_f32; 256]);
/// let shader = gpu.create_shader_module("
///     @group(0) @binding(0) var<storage, read_write> numbers: array<f32>;
///
///     @compute @workgroup_size(64) fn double(@builtin(global_invocation_id) id: vec3u) {
///         numbers[id.x] *= 2.0;
///     }
/// ");
/// let doubling = gpu.create_compute_pipeline(&shader, "double");
/// let group = gpu.create_bind_group(&doubling, 0, &[BindGroupEntry {
///     binding: 0,
///     resource: BindingResource::Buffer(&numbers),
/// }]);
///
/// let mut pass = gpu.begin_compute_pass();
/// pass.set_pipeline(&doubling);
/// pass.set_bind_group(0, &group);
/// pass.dispatch_workgroups(256 / 64, 1, 1);
/// ```
pub struct ComputePass<'a> {
    gpu: &'a WebGpu,
}

impl ComputePass<'_> {
    pub fn set_pipeline(&mut self, pipeline: &ComputePipeline) {
        self.gpu
            .op(code::SET_PIPELINE, |out| put_u32(out, pipeline.0.id));
    }

    /// The bind group of the shader's `@group(index)`.
    pub fn set_bind_group(&mut self, index: u32, group: &BindGroup) {
        set_bind_group(self.gpu, index, group);
    }

    /// `x` by `y` by `z` workgroups of the pipeline's entry point.
    pub fn dispatch_workgroups(&mut self, x: u32, y: u32, z: u32) {
        self.gpu.op(code::DISPATCH_WORKGROUPS, |out| {
            for value in [x, y, z] {
                put_u32(out, value);
            }
        });
    }

    /// Ends the pass, as dropping it does.
    pub fn end(self) {}
}

impl Drop for ComputePass<'_> {
    fn drop(&mut self) {
        self.gpu.op(code::END_PASS, |_| {});
    }
}

impl fmt::Debug for ComputePass<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ComputePass").finish_non_exhaustive()
    }
}

fn set_bind_group(gpu: &WebGpu, index: u32, group: &BindGroup) {
    gpu.op(code::SET_BIND_GROUP, |out| {
        put_u32(out, index);
        put_u32(out, group.0.id);
    });
}

/// See [`WebGpu::create_texture`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextureDescriptor {
    pub size: Size,
    pub format: TextureFormat,
    pub usage: TextureUsages,
}

/// See [`WebGpu::create_sampler`]. By default, the nearest pixel, and the edge outside.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SamplerDescriptor {
    pub address_mode_u: AddressMode,
    pub address_mode_v: AddressMode,
    pub mag_filter: FilterMode,
    pub min_filter: FilterMode,
    pub mipmap_filter: FilterMode,
}

/// See [`WebGpu::create_render_pipeline`].
#[derive(Clone, Copy, Debug)]
pub struct RenderPipelineDescriptor<'a> {
    pub vertex: VertexState<'a>,
    pub primitive: PrimitiveState,
    /// Whether the pipeline tests and writes depth, of the pass's depth attachment.
    pub depth_stencil: Option<DepthStencilState>,
    /// What colors the fragments get, if any: a pipeline only writing depth needs none.
    pub fragment: Option<FragmentState<'a>>,
}

impl RenderPipelineDescriptor<'_> {
    fn put(&self, out: &mut Vec<u8>) {
        let VertexState {
            module,
            entry_point,
            buffers,
        } = self.vertex;
        put_u32(out, module.0.id);
        put_source(out, entry_point);
        out.push(buffers.len() as u8);
        for buffer in buffers {
            put_u32(out, buffer.array_stride);
            out.push(buffer.step_mode as u8);
            out.push(buffer.attributes.len() as u8);
            for attribute in buffer.attributes {
                out.push(attribute.format as u8);
                put_u32(out, attribute.offset);
                put_u32(out, attribute.shader_location);
            }
        }

        let PrimitiveState {
            topology,
            strip_index_format,
            front_face,
            cull_mode,
        } = self.primitive;
        out.extend([
            topology as u8,
            strip_index_format.map_or(0, |format| format as u8 + 1),
            front_face as u8,
            cull_mode.map_or(0, |face| face as u8 + 1),
        ]);

        match self.depth_stencil {
            Some(depth) => out.extend([
                1,
                depth.format as u8,
                u8::from(depth.depth_write_enabled),
                depth.depth_compare as u8,
            ]),
            None => out.push(0),
        }

        match self.fragment {
            Some(fragment) => {
                out.push(1);
                put_u32(out, fragment.module.0.id);
                put_source(out, fragment.entry_point);
                out.push(fragment.targets.len() as u8);
                for target in fragment.targets {
                    out.push(target.format as u8);
                    match target.blend {
                        Some(blend) => {
                            out.push(1);
                            for component in [blend.color, blend.alpha] {
                                out.extend([
                                    component.src_factor as u8,
                                    component.dst_factor as u8,
                                    component.operation as u8,
                                ]);
                            }
                        }
                        None => out.push(0),
                    }
                    out.push(target.write_mask.0);
                }
            }
            None => out.push(0),
        }
    }
}

/// The vertex shader of a pipeline: the entry point `entry_point` of `module`, and the buffers
/// of its vertices, by slots.
#[derive(Clone, Copy, Debug)]
pub struct VertexState<'a> {
    pub module: &'a ShaderModule,
    pub entry_point: &'a str,
    pub buffers: &'a [VertexBufferLayout<'a>],
}

/// What a vertex buffer has: the attributes of each vertex, or of each instance, `array_stride`
/// bytes apart.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VertexBufferLayout<'a> {
    pub array_stride: u32,
    pub step_mode: VertexStepMode,
    pub attributes: &'a [VertexAttribute],
}

/// An attribute of a vertex, `offset` bytes into it, which the shader has at
/// `@location(shader_location)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VertexAttribute {
    pub format: VertexFormat,
    pub offset: u32,
    pub shader_location: u32,
}

/// What vertices make. By default, triangles, none of them left out.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PrimitiveState {
    pub topology: PrimitiveTopology,
    /// Of the index buffer of strips, whose largest index starts a new strip.
    pub strip_index_format: Option<IndexFormat>,
    pub front_face: FrontFace,
    /// Which triangles are left out: those showing their back, f. ex.
    pub cull_mode: Option<Face>,
}

/// The depth test of a pipeline.
///
/// ```rust
/// use vertigo::webgpu::{CompareFunction, DepthStencilState, TextureFormat};
///
/// // What is nearer hides what is behind.
/// let depth = DepthStencilState {
///     format: TextureFormat::Depth24Plus,
///     depth_write_enabled: true,
///     depth_compare: CompareFunction::Less,
/// };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DepthStencilState {
    /// Of the depth attachment.
    pub format: TextureFormat,
    pub depth_write_enabled: bool,
    pub depth_compare: CompareFunction,
}

/// The fragment shader of a pipeline: the entry point `entry_point` of `module`, and the color
/// attachments it writes, `@location(0)` first.
#[derive(Clone, Copy, Debug)]
pub struct FragmentState<'a> {
    pub module: &'a ShaderModule,
    pub entry_point: &'a str,
    pub targets: &'a [ColorTargetState],
}

/// A color attachment as a pipeline writes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorTargetState {
    pub format: TextureFormat,
    /// How what is drawn mixes with what is there. `None` replaces it.
    pub blend: Option<BlendState>,
    pub write_mask: ColorWrites,
}

impl ColorTargetState {
    /// The canvas, its colors replaced.
    pub fn canvas() -> Self {
        Self {
            format: TextureFormat::Canvas,
            blend: None,
            write_mask: ColorWrites::ALL,
        }
    }
}

/// How the colors and the alpha of what is drawn mix with those there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlendState {
    pub color: BlendComponent,
    pub alpha: BlendComponent,
}

impl BlendState {
    /// Over what is there, as much as the alpha of what is drawn.
    pub const ALPHA_BLENDING: Self = Self {
        color: BlendComponent {
            src_factor: BlendFactor::SrcAlpha,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        },
        alpha: BlendComponent {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        },
    };

    /// Over what is there, the colors drawn multiplied by their alpha already.
    pub const PREMULTIPLIED_ALPHA_BLENDING: Self = Self {
        color: BlendComponent {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        },
        alpha: BlendComponent {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        },
    };
}

/// `operation` on what is drawn times `src_factor`, and what is there times `dst_factor`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlendComponent {
    pub src_factor: BlendFactor,
    pub dst_factor: BlendFactor,
    pub operation: BlendOperation,
}

/// See [`WebGpu::create_bind_group`].
#[derive(Clone, Copy, Debug)]
pub struct BindGroupEntry<'a> {
    pub binding: u32,
    pub resource: BindingResource<'a>,
}

/// What a binding of a bind group is.
#[derive(Clone, Copy, Debug)]
pub enum BindingResource<'a> {
    /// All of a buffer.
    Buffer(&'a Buffer),
    /// `size` bytes of a buffer, from `offset`, a multiple of 256.
    BufferRange {
        buffer: &'a Buffer,
        offset: u32,
        size: u32,
    },
    Texture(&'a Texture),
    Sampler(&'a Sampler),
}

/// See [`WebGpu::begin_render_pass`].
#[derive(Clone, Copy, Debug)]
pub struct RenderPassDescriptor<'a> {
    pub color_attachments: &'a [ColorAttachment<'a>],
    pub depth_stencil_attachment: Option<DepthAttachment<'a>>,
}

impl RenderPassDescriptor<'_> {
    fn put(&self, out: &mut Vec<u8>) {
        out.push(self.color_attachments.len() as u8);
        for attachment in self.color_attachments {
            match attachment.target {
                Target::Canvas => out.push(0),
                Target::Texture(texture) => {
                    out.push(1);
                    put_u32(out, texture.object.id);
                }
            }
            match attachment.load {
                LoadOp::Load => out.push(0),
                LoadOp::Clear(Color { r, g, b, a }) => {
                    out.push(1);
                    for value in [r, g, b, a] {
                        out.extend_from_slice(&value.to_le_bytes());
                    }
                }
            }
            out.push(attachment.store as u8);
        }

        match self.depth_stencil_attachment {
            Some(attachment) => {
                out.push(1);
                match attachment.target {
                    DepthTarget::Canvas(format) => out.extend([0, format as u8]),
                    DepthTarget::Texture(texture) => {
                        out.push(1);
                        put_u32(out, texture.object.id);
                    }
                }
                match attachment.load {
                    LoadOp::Load => out.push(0),
                    LoadOp::Clear(depth) => {
                        out.push(1);
                        put_f32s(out, &[depth]);
                    }
                }
                out.push(attachment.store as u8);
            }
            None => out.push(0),
        }
    }
}

/// A texture a render pass draws colors into.
#[derive(Clone, Copy, Debug)]
pub struct ColorAttachment<'a> {
    pub target: Target<'a>,
    pub load: LoadOp<Color>,
    pub store: StoreOp,
}

impl ColorAttachment<'_> {
    /// The canvas's picture, kept.
    pub fn canvas(load: LoadOp<Color>) -> Self {
        Self {
            target: Target::Canvas,
            load,
            store: StoreOp::Store,
        }
    }
}

/// Where colors go.
#[derive(Clone, Copy, Debug)]
pub enum Target<'a> {
    /// The canvas's picture, in [`TextureFormat::Canvas`], shown once the batch is drawn.
    Canvas,
    Texture(&'a Texture),
}

/// The depths a render pass tests what it draws against.
///
/// ```rust
/// use vertigo::webgpu::{DepthAttachment, LoadOp, TextureFormat};
///
/// let depth = DepthAttachment::canvas(TextureFormat::Depth24Plus, LoadOp::Clear(1.0));
/// ```
#[derive(Clone, Copy, Debug)]
pub struct DepthAttachment<'a> {
    pub target: DepthTarget<'a>,
    pub load: LoadOp<f32>,
    pub store: StoreOp,
}

impl DepthAttachment<'_> {
    /// A depth texture as big as the canvas, of `format`, kept.
    pub fn canvas(format: TextureFormat, load: LoadOp<f32>) -> Self {
        Self {
            target: DepthTarget::Canvas(format),
            load,
            store: StoreOp::Store,
        }
    }
}

/// Where depths go.
#[derive(Clone, Copy, Debug)]
pub enum DepthTarget<'a> {
    /// A texture of the format, as big as the canvas's picture, made by the browser for the
    /// context and made again when the canvas changes its size: as the depth buffer of a WebGL
    /// canvas.
    Canvas(TextureFormat),
    Texture(&'a Texture),
}

/// What an attachment has when a render pass begins.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LoadOp<V> {
    /// What it had.
    Load,
    /// The value all over.
    Clear(V),
}

/// What an attachment keeps when a render pass ends.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum StoreOp {
    /// What was drawn.
    #[default]
    Store = 0,
    /// Nothing: what was drawn was needed only during the pass.
    Discard = 1,
}

/// Red, green, blue and alpha, from 0 to 1.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Color {
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
}

macro_rules! flags {
    ($(#[$doc:meta])* $name:ident($kind:ty) { $($(#[$flag_doc:meta])* $flag:ident = $value:expr),* $(,)? }) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
        pub struct $name($kind);

        impl $name {
            $($(#[$flag_doc])* pub const $flag: Self = Self($value);)*
        }

        impl BitOr for $name {
            type Output = Self;

            fn bitor(self, other: Self) -> Self {
                Self(self.0 | other.0)
            }
        }
    };
}

flags!(
    /// What a buffer is for, joined with `|`, as WebGPU's `GPUBufferUsage`.
    ///
    /// ```rust
    /// use vertigo::webgpu::BufferUsages;
    ///
    /// let uniforms = BufferUsages::UNIFORM | BufferUsages::COPY_DST;
    /// ```
    BufferUsages(u32) {
        MAP_READ = 0x0001,
        MAP_WRITE = 0x0002,
        COPY_SRC = 0x0004,
        /// Written into, f. ex. by [`WebGpu::write_buffer`].
        COPY_DST = 0x0008,
        INDEX = 0x0010,
        VERTEX = 0x0020,
        UNIFORM = 0x0040,
        STORAGE = 0x0080,
        INDIRECT = 0x0100,
        QUERY_RESOLVE = 0x0200,
    }
);

flags!(
    /// What a texture is for, joined with `|`, as WebGPU's `GPUTextureUsage`.
    TextureUsages(u32) {
        COPY_SRC = 0x01,
        /// Written into, f. ex. by [`WebGpu::write_texture`].
        COPY_DST = 0x02,
        /// Read by shaders, through a sampler.
        TEXTURE_BINDING = 0x04,
        STORAGE_BINDING = 0x08,
        /// Drawn into, by a render pass.
        RENDER_ATTACHMENT = 0x10,
    }
);

flags!(
    /// Which channels a pipeline writes, joined with `|`.
    ColorWrites(u8) {
        RED = 0x1,
        GREEN = 0x2,
        BLUE = 0x4,
        ALPHA = 0x8,
        COLOR = 0x7,
        ALL = 0xF,
    }
);

/// The formats of textures, as WebGPU's `GPUTextureFormat`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum TextureFormat {
    /// The one the browser draws canvases in fastest, `navigator.gpu.getPreferredCanvasFormat()`:
    /// `Bgra8Unorm` or `Rgba8Unorm`, of 4 bytes a pixel.
    Canvas = 0,
    Rgba8Unorm = 1,
    Rgba8UnormSrgb = 2,
    Bgra8Unorm = 3,
    Bgra8UnormSrgb = 4,
    R8Unorm = 5,
    Rg8Unorm = 6,
    R16Float = 7,
    Rg16Float = 8,
    Rgba16Float = 9,
    R32Float = 10,
    Rg32Float = 11,
    Rgba32Float = 12,
    R32Uint = 13,
    Depth16Unorm = 14,
    Depth24Plus = 15,
    Depth24PlusStencil8 = 16,
    Depth32Float = 17,
}

impl TextureFormat {
    /// Bytes of a pixel, as [`WebGpu::write_texture`] takes them. Depth of 24 bits takes 4.
    pub fn bytes_per_pixel(self) -> u32 {
        match self {
            Self::R8Unorm => 1,
            Self::Rg8Unorm | Self::R16Float | Self::Depth16Unorm => 2,
            Self::Rg16Float | Self::Rg32Float => 8,
            Self::Rgba16Float => 8,
            Self::Rgba32Float => 16,
            Self::Canvas
            | Self::Rgba8Unorm
            | Self::Rgba8UnormSrgb
            | Self::Bgra8Unorm
            | Self::Bgra8UnormSrgb
            | Self::R32Float
            | Self::R32Uint
            | Self::Depth24Plus
            | Self::Depth24PlusStencil8
            | Self::Depth32Float => 4,
        }
    }
}

/// The formats of vertex attributes, as WebGPU's `GPUVertexFormat`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum VertexFormat {
    Float32 = 0,
    Float32x2 = 1,
    Float32x3 = 2,
    Float32x4 = 3,
    Uint32 = 4,
    Uint32x2 = 5,
    Uint32x3 = 6,
    Uint32x4 = 7,
    Sint32 = 8,
    Sint32x2 = 9,
    Sint32x3 = 10,
    Sint32x4 = 11,
    /// 4 bytes, each from 0 to 1, f. ex. of a color.
    Unorm8x4 = 12,
    Uint8x4 = 13,
    Snorm8x4 = 14,
    Float16x2 = 15,
    Float16x4 = 16,
    Uint16x2 = 17,
    Unorm16x2 = 18,
}

/// Whether a vertex buffer has the attributes of each vertex or of each instance.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum VertexStepMode {
    #[default]
    Vertex = 0,
    Instance = 1,
}

/// What vertices make.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum PrimitiveTopology {
    PointList = 0,
    LineList = 1,
    LineStrip = 2,
    #[default]
    TriangleList = 3,
    TriangleStrip = 4,
}

/// The size of indices.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum IndexFormat {
    Uint16 = 0,
    Uint32 = 1,
}

/// Which way round the corners of a triangle facing the camera go, as the screen shows them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum FrontFace {
    #[default]
    Ccw = 0,
    Cw = 1,
}

/// A side of triangles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Face {
    Front = 0,
    Back = 1,
}

/// How a value is compared with what is there, f. ex. the depth of a fragment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum CompareFunction {
    Never = 0,
    Less = 1,
    Equal = 2,
    LessEqual = 3,
    Greater = 4,
    NotEqual = 5,
    GreaterEqual = 6,
    Always = 7,
}

/// What a color is multiplied by when it is blended, see [`BlendComponent`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum BlendFactor {
    Zero = 0,
    One = 1,
    Src = 2,
    OneMinusSrc = 3,
    SrcAlpha = 4,
    OneMinusSrcAlpha = 5,
    Dst = 6,
    OneMinusDst = 7,
    DstAlpha = 8,
    OneMinusDstAlpha = 9,
    SrcAlphaSaturated = 10,
    Constant = 11,
    OneMinusConstant = 12,
}

/// How a color drawn and the one there make one, see [`BlendComponent`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum BlendOperation {
    #[default]
    Add = 0,
    Subtract = 1,
    ReverseSubtract = 2,
    Min = 3,
    Max = 4,
}

/// How a texture is read outside of it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum AddressMode {
    #[default]
    ClampToEdge = 0,
    Repeat = 1,
    MirrorRepeat = 2,
}

/// How a texture is read between its pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum FilterMode {
    /// The nearest pixel: sharp, f. ex. for pixel art.
    #[default]
    Nearest = 0,
    /// Between the nearest pixels.
    Linear = 1,
}

fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_f32s(out: &mut Vec<u8>, values: &[f32]) {
    for value in values {
        out.extend_from_slice(&value.to_le_bytes());
    }
}

fn put_data(out: &mut Vec<u8>, data: &[u8]) {
    put_u32(out, data.len() as u32);
    out.extend_from_slice(data);
}

fn put_source(out: &mut Vec<u8>, source: &str) {
    put_u32(out, source.len() as u32);
    out.extend_from_slice(source.as_bytes());
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;
    use crate::{
        JsJsonNumber,
        dev::{
            CallbackId,
            inspect::{log_start, log_take},
        },
        dom_element, transaction,
    };

    impl WebGpu {
        /// The calls not sent yet, taken.
        fn take(&self) -> Vec<u8> {
            self.context.queue.change(std::mem::take)
        }
    }

    /// One call of each. A test runs on a thread of its own, so the objects' ids are 1, 2, ... in
    /// the order they are made.
    fn fixture() -> Vec<u8> {
        let canvas = dom_element! { <canvas /> };
        let gpu = WebGpu::with_options(
            &canvas.get_ref(),
            WebGpuOptions {
                alpha_mode: AlphaMode::Premultiplied,
                power_preference: PowerPreference::HighPerformance,
            },
        );
        let buffer = gpu.create_buffer(6, BufferUsages::VERTEX | BufferUsages::COPY_DST);
        gpu.write_buffer(&buffer, 4, &[1.5_f32, -2.0]);
        gpu.write_buffer(&buffer, 0, &[7_u16]);
        let texture = gpu.create_texture(&TextureDescriptor {
            size: Size {
                width: 2,
                height: 1,
            },
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        });
        gpu.write_texture(&texture, &[255, 0, 128, 255, 1, 2, 3, 4]);
        let sampler = gpu.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::MirrorRepeat,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Linear,
        });
        let shader = gpu.create_shader_module("wgsł");
        let pipeline = gpu.create_render_pipeline(&RenderPipelineDescriptor {
            vertex: VertexState {
                module: &shader,
                entry_point: "vs",
                buffers: &[
                    VertexBufferLayout {
                        array_stride: 24,
                        step_mode: VertexStepMode::Vertex,
                        attributes: &[
                            VertexAttribute {
                                format: VertexFormat::Float32x3,
                                offset: 0,
                                shader_location: 0,
                            },
                            VertexAttribute {
                                format: VertexFormat::Unorm8x4,
                                offset: 12,
                                shader_location: 1,
                            },
                        ],
                    },
                    VertexBufferLayout {
                        array_stride: 64,
                        step_mode: VertexStepMode::Instance,
                        attributes: &[VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: 16,
                            shader_location: 3,
                        }],
                    },
                ],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                strip_index_format: Some(IndexFormat::Uint32),
                front_face: FrontFace::Cw,
                cull_mode: Some(Face::Back),
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: CompareFunction::LessEqual,
            }),
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs",
                targets: &[
                    ColorTargetState::canvas(),
                    ColorTargetState {
                        format: TextureFormat::Rgba16Float,
                        blend: Some(BlendState::ALPHA_BLENDING),
                        write_mask: ColorWrites::COLOR,
                    },
                ],
            }),
        });
        let depth_only = gpu.create_render_pipeline(&RenderPipelineDescriptor {
            vertex: VertexState {
                module: &shader,
                entry_point: "vs",
                buffers: &[],
            },
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            fragment: None,
        });
        let compute = gpu.create_compute_pipeline(&shader, "cs");
        let group = gpu.create_bind_group(
            &pipeline,
            1,
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(&buffer),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::BufferRange {
                        buffer: &buffer,
                        offset: 256,
                        size: 64,
                    },
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Texture(&texture),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        );
        {
            let mut pass = gpu.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[
                    ColorAttachment::canvas(LoadOp::Clear(Color {
                        r: 0.0,
                        g: 0.25,
                        b: 0.5,
                        a: 1.0,
                    })),
                    ColorAttachment {
                        target: Target::Texture(&texture),
                        load: LoadOp::Load,
                        store: StoreOp::Discard,
                    },
                ],
                depth_stencil_attachment: Some(DepthAttachment::canvas(
                    TextureFormat::Depth24Plus,
                    LoadOp::Clear(1.0),
                )),
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(1, &group);
            pass.set_vertex_buffer(1, &buffer, 8);
            pass.set_index_buffer(&buffer, IndexFormat::Uint16, 2);
            pass.draw(1..4, 0..10);
            pass.draw_indexed(6..42, -1, 2..402);
            pass.set_viewport(0.0, 1.0, 640.0, 480.0, 0.0, 1.0);
            pass.set_scissor_rect(1, 2, 3, 4);
        }
        gpu.begin_render_pass(&RenderPassDescriptor {
            color_attachments: &[],
            depth_stencil_attachment: Some(DepthAttachment {
                target: DepthTarget::Texture(&texture),
                load: LoadOp::Load,
                store: StoreOp::Discard,
            }),
        })
        .end();
        {
            let mut pass = gpu.begin_compute_pass();
            pass.set_pipeline(&compute);
            pass.set_bind_group(0, &group);
            pass.dispatch_workgroups(4, 2, 1);
        }
        drop((
            buffer, texture, sampler, shader, pipeline, depth_only, compute, group,
        ));
        gpu.take()
    }

    /// The same bytes are played back in `driver_module/src_js/api/command/webgpu.test.ts`:
    /// change the format and one of the two tests fails, whichever side was changed.
    const FIXTURE: &[u8] = &[
        1, 1, 2, 1, 0, 0, 0, 0, 0, 0, 0, 2, 1, 0, 0, 0, 8, 0, 0, 0, 40, 0, 0, 0, 4, 1, 0, 0, 0, 4,
        0, 0, 0, 8, 0, 0, 0, 0, 0, 192, 63, 0, 0, 0, 192, 4, 1, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 7,
        0, 0, 0, 5, 2, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 1, 6, 0, 0, 0, 7, 2, 0, 0, 0, 8, 0, 0, 0,
        2, 0, 0, 0, 1, 0, 0, 0, 8, 0, 0, 0, 255, 0, 128, 255, 1, 2, 3, 4, 8, 3, 0, 0, 0, 1, 2, 1,
        0, 1, 10, 4, 0, 0, 0, 5, 0, 0, 0, 119, 103, 115, 197, 130, 12, 5, 0, 0, 0, 4, 0, 0, 0, 2,
        0, 0, 0, 118, 115, 2, 24, 0, 0, 0, 0, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 12, 12, 0, 0, 0, 1, 0,
        0, 0, 64, 0, 0, 0, 1, 1, 3, 16, 0, 0, 0, 3, 0, 0, 0, 4, 2, 1, 2, 1, 15, 1, 3, 1, 4, 0, 0,
        0, 2, 0, 0, 0, 102, 115, 2, 0, 0, 15, 9, 1, 4, 5, 0, 1, 5, 0, 7, 12, 6, 0, 0, 0, 4, 0, 0,
        0, 2, 0, 0, 0, 118, 115, 0, 3, 0, 0, 0, 0, 0, 13, 7, 0, 0, 0, 4, 0, 0, 0, 2, 0, 0, 0, 99,
        115, 15, 8, 0, 0, 0, 5, 0, 0, 0, 1, 0, 0, 0, 4, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 64, 0, 0, 0, 2, 0, 0, 0, 1, 2, 0, 0, 0, 3,
        0, 0, 0, 2, 3, 0, 0, 0, 17, 2, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 208, 63, 0,
        0, 0, 0, 0, 0, 224, 63, 0, 0, 0, 0, 0, 0, 240, 63, 0, 1, 2, 0, 0, 0, 0, 1, 1, 0, 15, 1, 0,
        0, 128, 63, 0, 20, 5, 0, 0, 0, 21, 1, 0, 0, 0, 8, 0, 0, 0, 22, 1, 0, 0, 0, 1, 0, 0, 0, 8,
        0, 0, 0, 23, 1, 0, 0, 0, 0, 2, 0, 0, 0, 24, 3, 0, 0, 0, 10, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
        0, 25, 36, 0, 0, 0, 144, 1, 0, 0, 6, 0, 0, 0, 255, 255, 255, 255, 2, 0, 0, 0, 27, 0, 0, 0,
        0, 0, 0, 128, 63, 0, 0, 32, 68, 0, 0, 240, 67, 0, 0, 0, 0, 0, 0, 128, 63, 28, 1, 0, 0, 0,
        2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 19, 17, 0, 1, 1, 2, 0, 0, 0, 0, 1, 19, 18, 20, 7, 0, 0,
        0, 21, 0, 0, 0, 0, 8, 0, 0, 0, 26, 4, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 19, 3, 1, 0, 0, 0,
        6, 2, 0, 0, 0, 9, 3, 0, 0, 0, 11, 4, 0, 0, 0, 14, 5, 0, 0, 0, 14, 6, 0, 0, 0, 14, 7, 0, 0,
        0, 16, 8, 0, 0, 0,
    ];

    #[test]
    fn matches_the_cross_language_fixture() {
        assert_eq!(
            fixture(),
            FIXTURE,
            "the format changed - update webgpu.test.ts to match, or this is a bug"
        );
    }

    /// The `WebGpu` commands of a batch, with the canvases created before them.
    fn batch(run: impl FnOnce()) -> Vec<(&'static str, Vec<u8>)> {
        log_start();
        transaction(|_| run());
        log_take()
            .into_iter()
            .filter_map(|command| match command {
                DriverDomCommand::CreateNode { name, .. } if name.as_str() == "canvas" => {
                    Some(("canvas", Vec::new()))
                }
                DriverDomCommand::WebGpu { commands, .. } => Some(("webgpu", commands)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn the_calls_of_a_transaction_follow_their_canvas_in_one_command() {
        let mut gpu = None;
        let sent = batch(|| {
            let canvas = dom_element! { <canvas /> };
            let context = WebGpu::new(&canvas.get_ref());
            context.begin_compute_pass().dispatch_workgroups(1, 1, 1);
            gpu = Some((canvas, context));
        });

        let mut calls = vec![code::CONTEXT, 0, 0];
        // The context's callback, the first of the test's thread
        calls.extend(1_u64.to_le_bytes());
        calls.push(code::BEGIN_COMPUTE_PASS);
        calls.extend([
            code::DISPATCH_WORKGROUPS,
            1,
            0,
            0,
            0,
            1,
            0,
            0,
            0,
            1,
            0,
            0,
            0,
        ]);
        calls.push(code::END_PASS);
        assert_eq!(sent, vec![("canvas", vec![]), ("webgpu", calls)]);

        assert_eq!(batch(|| {}), vec![], "nothing more to send");
    }

    /// Tells `gpu`'s context `values`, as the browser does.
    fn teller(gpu: &WebGpu) -> impl Fn(&[u32]) {
        let calls = gpu.take();
        let Ok(callback) = <[u8; 8]>::try_from(&calls[3..11]) else {
            panic!("a context starts with its callback: {calls:?}");
        };
        let callback = CallbackId::from_u64(u64::from_le_bytes(callback));
        move |values: &[u32]| {
            let values = values
                .iter()
                .map(|value| JsJson::Number(JsJsonNumber(f64::from(*value))));
            api_callbacks().call(callback, JsJson::List(values.collect()));
        }
    }

    #[test]
    fn the_browser_tells_the_size_and_the_device_given_back() {
        let canvas = dom_element! { <canvas /> };
        let gpu = WebGpu::new(&canvas.get_ref());
        let tell = teller(&gpu);

        let sizes = Rc::new(RefCell::new(Vec::new()));
        let restored = Rc::new(Cell::new(0));
        let _resized = gpu.on_resize({
            let sizes = sizes.clone();
            move |size| sizes.borrow_mut().push(size)
        });
        let _restored = gpu.on_restore({
            let restored = restored.clone();
            move || restored.set(restored.get() + 1)
        });
        assert_eq!(
            gpu.size(),
            Size::default(),
            "no size until the browser tells it"
        );

        tell(&[0, 640, 480]);
        let size = Size {
            width: 640,
            height: 480,
        };
        assert_eq!(gpu.size(), size);
        assert_eq!(*sizes.borrow(), vec![size]);

        tell(&[1]);
        assert_eq!(restored.get(), 1);
    }

    #[test]
    fn a_browser_without_webgpu_is_told_also_to_those_asking_later() {
        let canvas = dom_element! { <canvas /> };
        let gpu = WebGpu::new(&canvas.get_ref());
        let tell = teller(&gpu);

        let told = Rc::new(Cell::new(0));
        let count = {
            let told = told.clone();
            move || told.set(told.get() + 1)
        };
        let _before = gpu.on_unavailable(count.clone());
        assert_eq!(told.get(), 0, "not until the browser tells it");

        tell(&[2]);
        assert_eq!(told.get(), 1);
        let _after = gpu.on_unavailable(count);
        assert_eq!(told.get(), 2, "told at once, once known");
    }

    #[test]
    fn an_object_dropped_is_let_go_and_a_context_dropped_sends_nothing() {
        let canvas = dom_element! { <canvas /> };
        let gpu = WebGpu::new(&canvas.get_ref());
        let buffer = gpu.create_buffer(4, BufferUsages::UNIFORM);
        batch(|| {});

        let sent = batch(|| drop(buffer));
        assert_eq!(
            sent,
            vec![("webgpu", vec![code::DESTROY_BUFFER, 1, 0, 0, 0])]
        );

        let shader = gpu.create_shader_module("");
        drop(gpu);
        assert_eq!(batch(|| drop(shader)), vec![], "the context is gone");
    }

    #[test]
    fn data_is_written_in_whole_words() {
        let canvas = dom_element! { <canvas /> };
        let gpu = WebGpu::new(&canvas.get_ref());
        gpu.take();

        let indices = gpu.create_buffer_init(BufferUsages::INDEX, &[1_u16, 2, 3]);
        assert_eq!(indices.size(), 8, "the size rounded up");
        let calls = gpu.take();
        assert_eq!(
            calls[calls.len() - 8..],
            [1, 0, 2, 0, 3, 0, 0, 0],
            "and the data filled up with zeros"
        );
    }
}
