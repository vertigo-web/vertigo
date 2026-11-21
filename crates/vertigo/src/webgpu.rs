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
//! source    := length:u32 utf8*           -- WGSL, an entry point's name, or a URL
//! data      := length:u32 byte*
//! enum      := u8                         -- the place of a value in the list of WebGPU's
//!                                            strings for it, see the enums here
//! pipeline  := layout:id vertex primitive depth fragment samples:u8  -- layout 0 is "auto"
//! vertex    := module:id entry:source count:u8 (stride:u32 step:enum count:u8
//!              (format:enum offset:u32 location:u32)*)*
//! primitive := topology:enum strip_index_format:enum front_face:enum cull_mode:enum
//! depth     := 0 | 1 format:enum write:u8 compare:enum
//! fragment  := 0 | 1 module:id entry:source count:u8 (format:enum blend write_mask:u8)*
//! blend     := 0 | 1 (src:enum dst:enum operation:enum){2}  -- of color, then of alpha
//! layout_entry := binding:u32 visibility:u8 kind:u8 multisampled:u8  -- kind: see `BindingType`
//! entry     := binding:u32 (0 buffer:id offset:u32 size:u32 | 1 texture:id | 2 sampler:id)
//! pass      := count:u8 (target resolve:(0 | 1 target) load:(0 | 1 r:f64 g:f64 b:f64 a:f64)
//!              store:u8)* (0 | 1 depth_target load:(0 | 1 depth:f32) store:u8)
//! target    := 0 | 1 texture:id | 2 samples:u8  -- the canvas, a texture, or a multisampled
//!                                                  one as big as the canvas
//! depth_target := 0 format:enum | 1 texture:id  -- one as big as the canvas, or a texture
//! ```
//!
//! A WebGPU flag, f. ex. of [`BufferUsages`], travels as its number, a `u32`, and a size of
//! 0 for a buffer's binding means the rest of it.

use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
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

pub use crate::webgl::{ImageLoad, Scalar, Size};

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
    pub const CREATE_BIND_GROUP_LAYOUT: u8 = 29;
    pub const DELETE_BIND_GROUP_LAYOUT: u8 = 30;
    pub const CREATE_PIPELINE_LAYOUT: u8 = 31;
    pub const DELETE_PIPELINE_LAYOUT: u8 = 32;
    /// The layout of a bind group WebGPU made for a pipeline of the layout "auto".
    pub const PIPELINE_BIND_GROUP_LAYOUT: u8 = 33;
    pub const GENERATE_MIPMAPS: u8 = 34;
    pub const TEXTURE_FROM_URL: u8 = 35;
    pub const READ_BUFFER: u8 = 36;
    pub const READ_PIXELS: u8 = 37;
    /// The scale of the canvas's bitmap, as `CONTEXT` has it.
    pub const BITMAP_SCALE: u8 = 38;
}

/// What the browser tells of a context, as `webgpu.ts` sends it: `[0, width, height]` when the
/// canvas changes its size, `[1]` when the context has a new device after it lost its own, `[2]`
/// when the browser has no WebGPU for it, `[3, texture, width, height]` when an image is in a
/// texture, `[4, texture]` when it didn't load, `[5, request, bytes]` with what a read read,
/// `[6, request, width, height, bytes]` with the pixels a read read.
enum GpuEvent {
    Resized(Size),
    Restored,
    Unavailable,
    ImageLoaded(u32, Size),
    ImageFailed(u32),
    Read(u32, Vec<u8>),
    Pixels(u32, Size, Vec<u8>),
}

impl GpuEvent {
    fn from_driver(data: JsJson) -> Result<Self, String> {
        data.map_list(|mut params: JsJsonListDecoder| {
            let size = |params: &mut JsJsonListDecoder| -> Result<Size, String> {
                Ok(Size {
                    width: params.get_u32("width")?,
                    height: params.get_u32("height")?,
                })
            };
            let event = match params.get_u32("kind")? {
                0 => Self::Resized(size(&mut params)?),
                1 => Self::Restored,
                2 => Self::Unavailable,
                3 => Self::ImageLoaded(params.get_u32("texture")?, size(&mut params)?),
                4 => Self::ImageFailed(params.get_u32("texture")?),
                5 => Self::Read(params.get_u32("request")?, params.get_buffer("bytes")?),
                6 => Self::Pixels(
                    params.get_u32("request")?,
                    size(&mut params)?,
                    params.get_buffer("bytes")?,
                ),
                other => return Err(format!("unknown gpu event {other}")),
            };
            params.expect_no_more()?;
            Ok(event)
        })
    }
}

/// What a read waits for, see [`WebGpu::read_buffer`] and [`WebGpu::read_pixels`].
enum Reading {
    Bytes(Box<dyn FnOnce(Vec<u8>)>),
    Pixels(Box<dyn FnOnce(Size, Vec<u8>)>),
}

/// A texture an image is coming into, see [`WebGpu::create_texture_from_url`].
struct Coming {
    url: String,
    /// Its size, which the image's becomes.
    size: Weak<Cell<Size>>,
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
    /// The images coming into textures, by the textures' ids.
    images: RefCell<HashMap<u32, Coming>>,
    image_loaded: EventEmitter<ImageLoad>,
    /// The reads not answered yet, by their numbers.
    reads: RefCell<HashMap<u32, Reading>>,
    last_read: Cell<u32>,
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
            put_f32s(&mut queue, &[options.bitmap_scale]);

            Self {
                queue: ValueMut::new(queue),
                size: Cell::new(Size::default()),
                resized: EventEmitter::default(),
                restored: EventEmitter::default(),
                unavailable: Cell::new(false),
                told_unavailable: EventEmitter::default(),
                images: RefCell::new(HashMap::new()),
                image_loaded: EventEmitter::default(),
                reads: RefCell::new(HashMap::new()),
                last_read: Cell::new(0),
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
            Ok(GpuEvent::Restored) => {
                // Gone with the device: no image comes, and no read is answered.
                self.images.borrow_mut().clear();
                self.reads.borrow_mut().clear();
                self.restored.trigger(&());
            }
            Ok(GpuEvent::Unavailable) => {
                self.unavailable.set(true);
                self.told_unavailable.trigger(&());
            }
            Ok(GpuEvent::ImageLoaded(texture, size)) => {
                let coming = self.images.borrow_mut().remove(&texture);
                if let Some(Coming { url, size: of }) = coming {
                    if let Some(of) = of.upgrade() {
                        of.set(size);
                    }
                    self.image_loaded.trigger(&ImageLoad::Loaded { url, size });
                }
            }
            Ok(GpuEvent::ImageFailed(texture)) => {
                let coming = self.images.borrow_mut().remove(&texture);
                if let Some(Coming { url, .. }) = coming {
                    self.image_loaded.trigger(&ImageLoad::Failed { url });
                }
            }
            Ok(GpuEvent::Read(request, bytes)) => {
                let reading = self.reads.borrow_mut().remove(&request);
                if let Some(Reading::Bytes(callback)) = reading {
                    callback(bytes);
                }
            }
            Ok(GpuEvent::Pixels(request, size, bytes)) => {
                let reading = self.reads.borrow_mut().remove(&request);
                if let Some(Reading::Pixels(callback)) = reading {
                    callback(size, bytes);
                }
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

/// The id of a new object.
fn next_id() -> u32 {
    LAST_ID.with(|last| {
        last.set(last.get() + 1);
        last.get()
    })
}

/// How the canvas gets its WebGPU context. The first [`WebGpu`] of a canvas sets them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WebGpuOptions {
    pub alpha_mode: AlphaMode,
    /// Which GPU the browser is asked for, where there are more.
    pub power_preference: PowerPreference,
    /// Pixels of the canvas's picture to a device pixel of the canvas, across and down: 1, so the
    /// picture is sharp, or less, f. ex. 0.5 with a quarter of the pixels to draw, which a slow
    /// GPU fills sooner, the browser stretching them over the canvas. Not above 0 nor finite counts
    /// as 1. See [`WebGlOptions::bitmap_scale`](crate::webgl::WebGlOptions::bitmap_scale).
    /// [`WebGpu::set_bitmap_scale`] changes it later.
    pub bitmap_scale: f32,
}

impl Default for WebGpuOptions {
    /// Opaque, on the GPU the browser chooses, sharp.
    fn default() -> Self {
        Self {
            alpha_mode: AlphaMode::default(),
            power_preference: PowerPreference::default(),
            bitmap_scale: 1.0,
        }
    }
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
///     FragmentState, LoadOp, RenderPassDescriptor, RenderPipelineDescriptor, VertexState, WebGpu,
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
///     let pipeline = gpu.create_render_pipeline(&RenderPipelineDescriptor::new(
///         VertexState { module: &shader, entry_point: "vertex", buffers: &[] },
///         Some(FragmentState {
///             module: &shader,
///             entry_point: "fragment",
///             targets: &[ColorTargetState::canvas()],
///         }),
///     ));
///     let color = gpu.create_buffer(16, BufferUsages::UNIFORM | BufferUsages::COPY_DST);
///     let colors = gpu.create_bind_group(&pipeline.get_bind_group_layout(0), &[BindGroupEntry {
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
    ///     // A quarter of the pixels, for a slow GPU
    ///     bitmap_scale: 0.5,
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

    /// Changes [`WebGpuOptions::bitmap_scale`] of the canvas, keeping its device and the objects
    /// made on it, which a new context would have to make again, f. ex. loading the textures. The
    /// canvas gets the size as from a resize: [`Self::on_resize`] tells it, and the passes that come
    /// next draw on a picture of that size.
    ///
    /// ```rust
    /// use vertigo::dom_element;
    /// use vertigo::webgpu::WebGpu;
    ///
    /// let canvas = dom_element! { <canvas /> };
    /// let gpu = WebGpu::new(&canvas.get_ref());
    /// // The frames come late: a quarter of the pixels from now on.
    /// gpu.set_bitmap_scale(0.5);
    /// ```
    pub fn set_bitmap_scale(&self, scale: f32) {
        self.op(code::BITMAP_SCALE, |out| put_f32s(out, &[scale]));
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
        let id = next_id();
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

    /// A 2D texture, see [`TextureDescriptor`], [`Self::write_texture`] and
    /// [`Self::generate_mipmaps`].
    pub fn create_texture(&self, descriptor: &TextureDescriptor) -> Texture {
        let TextureDescriptor {
            size,
            mip_level_count,
            sample_count,
            format,
            usage,
        } = *descriptor;
        let mip_level_count = mip_level_count.max(1);
        let sample_count = sample_count.max(1);
        Texture {
            object: self.object(code::CREATE_TEXTURE, code::DESTROY_TEXTURE, |out| {
                put_u32(out, size.width);
                put_u32(out, size.height);
                out.push(format as u8);
                put_u32(out, usage.0);
                put_u32(out, mip_level_count);
                out.push(sample_count as u8);
            }),
            size: Rc::new(Cell::new(size)),
            mip_level_count,
            sample_count,
            format,
        }
    }

    /// A texture of the image at `url`, which the browser loads: 1 x 1 white pixel until it comes,
    /// then as big as the image, with smaller images of it if [`ImageOptions::mipmaps`]. Bind
    /// groups with it take the image then. [`Self::on_image_load`] tells when it came, or didn't.
    /// An image from another origin needs CORS headers allowing it, as WebGPU reads its pixels.
    ///
    /// Of [`TextureFormat::Rgba8Unorm`], for [`TextureUsages::TEXTURE_BINDING`] and
    /// [`TextureUsages::COPY_SRC`]: drawn with, and read back.
    ///
    /// ```rust
    /// use vertigo::dom_element;
    /// use vertigo::webgpu::{ImageLoad, ImageOptions, WebGpu};
    ///
    /// let canvas = dom_element! { <canvas /> };
    /// let gpu = WebGpu::new(&canvas.get_ref());
    /// let bricks = gpu.create_texture_from_url("/images/bricks.png", &ImageOptions {
    ///     mipmaps: true,
    ///     ..ImageOptions::default()
    /// });
    ///
    /// canvas.append_drop_resource(gpu.on_image_load(|load| {
    ///     if let ImageLoad::Loaded { .. } = load {
    ///         // Draw again, with the bricks.
    ///     }
    /// }));
    /// ```
    pub fn create_texture_from_url(&self, url: &str, options: &ImageOptions) -> Texture {
        let size = Rc::new(Cell::new(Size {
            width: 1,
            height: 1,
        }));
        let object = self.object(code::TEXTURE_FROM_URL, code::DESTROY_TEXTURE, |out| {
            out.extend([
                u8::from(options.mipmaps),
                u8::from(options.premultiplied_alpha),
                u8::from(options.flip_y),
            ]);
            put_source(out, url);
        });
        self.context.images.borrow_mut().insert(
            object.id,
            Coming {
                url: url.to_string(),
                size: Rc::downgrade(&size),
            },
        );
        Texture {
            object,
            size,
            mip_level_count: 1,
            sample_count: 1,
            format: TextureFormat::Rgba8Unorm,
        }
    }

    /// `callback` when an image asked for with [`Self::create_texture_from_url`] is in its texture,
    /// or didn't load.
    pub fn on_image_load(&self, callback: impl Fn(&ImageLoad) + 'static) -> DropResource {
        self.context.image_loaded.add(move |load| callback(&load))
    }

    /// Writes all of `texture`'s pixels, of its first level, row after row from the top, each as
    /// its format has it, f. ex. 4 bytes of red, green, blue and alpha. The texture needs
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
    ///     mip_level_count: 1,
    ///     sample_count: 1,
    ///     format: TextureFormat::Rgba8Unorm,
    ///     usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
    /// });
    /// gpu.write_texture(&checker, &[
    ///     0, 0, 0, 255, 255, 255, 255, 255,
    ///     255, 255, 255, 255, 0, 0, 0, 255,
    /// ]);
    /// ```
    pub fn write_texture(&self, texture: &Texture, data: &[u8]) {
        let size = texture.size();
        self.op(code::WRITE_TEXTURE, |out| {
            put_u32(out, texture.object.id);
            put_u32(out, size.width * texture.format.bytes_per_pixel());
            put_u32(out, size.width);
            put_u32(out, size.height);
            put_data(out, data);
        });
    }

    /// Makes the smaller images of `texture`'s levels from its first, each half as big as the one
    /// before, smoothly: for a sampler with a mipmap filter, so the texture shown small is calm.
    /// The texture needs more levels than one, see [`Size::mip_levels`], and
    /// [`TextureUsages::TEXTURE_BINDING`] and [`TextureUsages::RENDER_ATTACHMENT`], as WebGPU
    /// draws them, and a format of colors read between pixels, f. ex.
    /// [`TextureFormat::Rgba8Unorm`].
    ///
    /// ```rust
    /// use vertigo::dom_element;
    /// use vertigo::webgpu::{Size, TextureDescriptor, TextureFormat, TextureUsages, WebGpu};
    ///
    /// let canvas = dom_element! { <canvas /> };
    /// let gpu = WebGpu::new(&canvas.get_ref());
    /// let size = Size { width: 64, height: 64 };
    /// let stripes = gpu.create_texture(&TextureDescriptor {
    ///     size,
    ///     // 64, 32, ... 1 pixels across
    ///     mip_level_count: size.mip_levels(),
    ///     sample_count: 1,
    ///     format: TextureFormat::Rgba8Unorm,
    ///     usage: TextureUsages::TEXTURE_BINDING
    ///         | TextureUsages::COPY_DST
    ///         | TextureUsages::RENDER_ATTACHMENT,
    /// });
    /// let pixels: Vec<u8> = (0..64 * 64)
    ///     .flat_map(|pixel| if pixel % 2 == 0 { [0, 0, 0, 255] } else { [255; 4] })
    ///     .collect();
    /// gpu.write_texture(&stripes, &pixels);
    /// // Grey, smaller
    /// gpu.generate_mipmaps(&stripes);
    /// ```
    pub fn generate_mipmaps(&self, texture: &Texture) {
        self.op(code::GENERATE_MIPMAPS, |out| {
            put_u32(out, texture.object.id)
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
        descriptor: &ComputePipelineDescriptor<'_>,
    ) -> ComputePipeline {
        let ComputePipelineDescriptor {
            layout,
            module,
            entry_point,
        } = *descriptor;
        ComputePipeline(self.object(
            code::CREATE_COMPUTE_PIPELINE,
            code::DELETE_PIPELINE,
            |out| {
                put_u32(out, layout.map_or(0, |layout| layout.0.id));
                put_u32(out, module.0.id);
                put_source(out, entry_point);
            },
        ))
    }

    /// What the bind groups of a `@group(...)` of shaders have, by their bindings, so that a bind
    /// group serves the pipelines of all layouts with it, see [`Self::create_pipeline_layout`].
    ///
    /// ```rust
    /// use vertigo::dom_element;
    /// use vertigo::webgpu::{
    ///     BindGroupLayoutEntry, BindingType, BufferBindingType, SamplerBindingType, ShaderStages,
    ///     TextureSampleType, WebGpu,
    /// };
    ///
    /// let canvas = dom_element! { <canvas /> };
    /// let gpu = WebGpu::new(&canvas.get_ref());
    /// // A uniform buffer for both shaders, and a texture with its sampler for the fragments'.
    /// let layout = gpu.create_bind_group_layout(&[
    ///     BindGroupLayoutEntry {
    ///         binding: 0,
    ///         visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
    ///         ty: BindingType::Buffer(BufferBindingType::Uniform),
    ///     },
    ///     BindGroupLayoutEntry {
    ///         binding: 1,
    ///         visibility: ShaderStages::FRAGMENT,
    ///         ty: BindingType::Texture {
    ///             sample_type: TextureSampleType::Float { filterable: true },
    ///             multisampled: false,
    ///         },
    ///     },
    ///     BindGroupLayoutEntry {
    ///         binding: 2,
    ///         visibility: ShaderStages::FRAGMENT,
    ///         ty: BindingType::Sampler(SamplerBindingType::Filtering),
    ///     },
    /// ]);
    /// let pipelines = gpu.create_pipeline_layout(&[&layout]);
    /// ```
    pub fn create_bind_group_layout(&self, entries: &[BindGroupLayoutEntry]) -> BindGroupLayout {
        BindGroupLayout(self.object(
            code::CREATE_BIND_GROUP_LAYOUT,
            code::DELETE_BIND_GROUP_LAYOUT,
            |out| {
                out.push(entries.len() as u8);
                for entry in entries {
                    put_u32(out, entry.binding);
                    out.push(entry.visibility.0);
                    let (kind, multisampled) = entry.ty.kind();
                    out.extend([kind, u8::from(multisampled)]);
                }
            },
        ))
    }

    /// The layouts of the bind groups of pipelines, `@group(0)` first, see
    /// [`RenderPipelineDescriptor::layout`].
    pub fn create_pipeline_layout(
        &self,
        bind_group_layouts: &[&BindGroupLayout],
    ) -> PipelineLayout {
        PipelineLayout(self.object(
            code::CREATE_PIPELINE_LAYOUT,
            code::DELETE_PIPELINE_LAYOUT,
            |out| {
                out.push(bind_group_layouts.len() as u8);
                for layout in bind_group_layouts {
                    put_u32(out, layout.0.id);
                }
            },
        ))
    }

    /// The resources of a bind group of `layout`, by their bindings: those of
    /// `@group(...) @binding(binding)` of shaders. See also
    /// [`RenderPipeline::get_bind_group_layout`].
    pub fn create_bind_group(
        &self,
        layout: &BindGroupLayout,
        entries: &[BindGroupEntry<'_>],
    ) -> BindGroup {
        BindGroup(
            self.object(code::CREATE_BIND_GROUP, code::DELETE_BIND_GROUP, |out| {
                put_u32(out, layout.0.id);
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

    /// `callback` with the bytes of `buffer`, as the passes before this left them, once the GPU
    /// has done them. The buffer needs [`BufferUsages::COPY_SRC`]. Nothing is told if the device
    /// is lost before.
    ///
    /// ```rust
    /// use vertigo::{Value, dom_element};
    /// use vertigo::webgpu::{BufferUsages, WebGpu};
    ///
    /// let canvas = dom_element! { <canvas /> };
    /// let gpu = WebGpu::new(&canvas.get_ref());
    /// let numbers = gpu.create_buffer_init(BufferUsages::STORAGE | BufferUsages::COPY_SRC, &[1.0_f32; 4]);
    /// // ... a compute pass doubling them
    /// let doubled = Value::new(Vec::new());
    /// gpu.read_buffer(&numbers, {
    ///     let doubled = doubled.clone();
    ///     move |bytes| doubled.set(bytes)
    /// });
    /// ```
    pub fn read_buffer(&self, buffer: &Buffer, callback: impl FnOnce(Vec<u8>) + 'static) {
        let request = self.reading(Reading::Bytes(Box::new(callback)));
        self.op(code::READ_BUFFER, |out| {
            put_u32(out, request);
            put_u32(out, buffer.object.id);
        });
    }

    /// `callback` with the size and the pixels of `target`, of its first level, as the passes
    /// before this drew them, once the GPU has drawn them: row after row from the top, each as the
    /// texture's format has it, and the canvas's as red, green, blue and alpha. A texture needs
    /// [`TextureUsages::COPY_SRC`]. The canvas has a picture only in a batch drawing on it, as
    /// the browser takes it once it is shown. Nothing is told if the device is lost before.
    ///
    /// ```rust
    /// use vertigo::{dom_element, log};
    /// use vertigo::webgpu::{Color, ColorAttachment, LoadOp, RenderPassDescriptor, Target, WebGpu};
    ///
    /// let canvas = dom_element! { <canvas /> };
    /// let gpu = WebGpu::new(&canvas.get_ref());
    /// gpu.begin_render_pass(&RenderPassDescriptor {
    ///     color_attachments: &[ColorAttachment::canvas(LoadOp::Clear(Color::WHITE))],
    ///     depth_stencil_attachment: None,
    /// });
    /// // A screenshot
    /// gpu.read_pixels(Target::Canvas, |size, pixels| {
    ///     log::info!("{} x {}: {} bytes", size.width, size.height, pixels.len());
    /// });
    /// ```
    pub fn read_pixels(&self, target: Target<'_>, callback: impl FnOnce(Size, Vec<u8>) + 'static) {
        let request = self.reading(Reading::Pixels(Box::new(callback)));
        self.op(code::READ_PIXELS, |out| {
            put_u32(out, request);
            target.put(out);
        });
    }

    /// The number of a read, which waits for its answer.
    fn reading(&self, reading: Reading) -> u32 {
        let request = self.context.last_read.get() + 1;
        self.context.last_read.set(request);
        self.context.reads.borrow_mut().insert(request, reading);
        request
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

/// A 2D texture, see [`WebGpu::create_texture`] and [`WebGpu::create_texture_from_url`].
#[derive(Debug)]
pub struct Texture {
    object: Object,
    /// One of an image becomes the image's when it comes.
    size: Rc<Cell<Size>>,
    mip_level_count: u32,
    sample_count: u32,
    format: TextureFormat,
}

impl Texture {
    /// Of its first level. One from a URL is 1 x 1 until its image comes.
    pub fn size(&self) -> Size {
        self.size.get()
    }

    pub fn mip_level_count(&self) -> u32 {
        self.mip_level_count
    }

    pub fn sample_count(&self) -> u32 {
        self.sample_count
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

impl RenderPipeline {
    /// The layout of its bind group `index`, `@group(index)` of its shaders: the one it was made
    /// with, or the one WebGPU found in its shaders for a pipeline without
    /// [`RenderPipelineDescriptor::layout`], which serves this pipeline only.
    ///
    /// ```rust
    /// use vertigo::dom_element;
    /// use vertigo::webgpu::{
    ///     BindGroupEntry, BindingResource, BufferUsages, ColorTargetState, FragmentState,
    ///     RenderPipelineDescriptor, VertexState, WebGpu,
    /// };
    ///
    /// let canvas = dom_element! { <canvas /> };
    /// let gpu = WebGpu::new(&canvas.get_ref());
    /// let shader = gpu.create_shader_module("
    ///     @group(0) @binding(0) var<uniform> color: vec4f;
    ///     @vertex fn vertex() -> @builtin(position) vec4f { return vec4f(0.0, 0.0, 0.0, 1.0); }
    ///     @fragment fn fragment() -> @location(0) vec4f { return color; }
    /// ");
    /// let pipeline = gpu.create_render_pipeline(&RenderPipelineDescriptor::new(
    ///     VertexState { module: &shader, entry_point: "vertex", buffers: &[] },
    ///     Some(FragmentState {
    ///         module: &shader,
    ///         entry_point: "fragment",
    ///         targets: &[ColorTargetState::canvas()],
    ///     }),
    /// ));
    /// let color = gpu.create_buffer(16, BufferUsages::UNIFORM | BufferUsages::COPY_DST);
    /// let group = gpu.create_bind_group(&pipeline.get_bind_group_layout(0), &[BindGroupEntry {
    ///     binding: 0,
    ///     resource: BindingResource::Buffer(&color),
    /// }]);
    /// ```
    pub fn get_bind_group_layout(&self, index: u32) -> BindGroupLayout {
        bind_group_layout_of(&self.0, index)
    }
}

/// See [`WebGpu::create_compute_pipeline`].
#[derive(Debug)]
pub struct ComputePipeline(Object);

impl ComputePipeline {
    /// The layout of its bind group `index`, see [`RenderPipeline::get_bind_group_layout`].
    pub fn get_bind_group_layout(&self, index: u32) -> BindGroupLayout {
        bind_group_layout_of(&self.0, index)
    }
}

/// The layout of the bind group `index` of `pipeline`, a new handle of it.
fn bind_group_layout_of(pipeline: &Object, index: u32) -> BindGroupLayout {
    let id = next_id();
    if let Some(context) = pipeline.context.upgrade() {
        context.queue.change(|out| {
            out.push(code::PIPELINE_BIND_GROUP_LAYOUT);
            put_u32(out, id);
            put_u32(out, pipeline.id);
            put_u32(out, index);
        });
    }
    BindGroupLayout(Object {
        id,
        delete: code::DELETE_BIND_GROUP_LAYOUT,
        context: pipeline.context.clone(),
    })
}

/// See [`WebGpu::create_bind_group_layout`].
#[derive(Debug)]
pub struct BindGroupLayout(Object);

/// See [`WebGpu::create_pipeline_layout`].
#[derive(Debug)]
pub struct PipelineLayout(Object);

/// See [`WebGpu::create_bind_group`].
#[derive(Debug)]
pub struct BindGroup(Object);

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
/// use vertigo::webgpu::{
///     BindGroupEntry, BindingResource, BufferUsages, ComputePipelineDescriptor, WebGpu,
/// };
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
/// let doubling = gpu.create_compute_pipeline(&ComputePipelineDescriptor {
///     layout: None,
///     module: &shader,
///     entry_point: "double",
/// });
/// let group = gpu.create_bind_group(&doubling.get_bind_group_layout(0), &[BindGroupEntry {
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
    /// Of its first level.
    pub size: Size,
    /// How many levels it has, each half as big as the one before, see [`Size::mip_levels`] and
    /// [`WebGpu::generate_mipmaps`]: 1, or 0, for the first only.
    pub mip_level_count: u32,
    /// Samples a pixel: 1, or 0, or more, f. ex. 4, for a pass smoothing edges, which it resolves
    /// into a texture of 1, see [`ColorAttachment::resolve_target`].
    pub sample_count: u32,
    pub format: TextureFormat,
    pub usage: TextureUsages,
}

/// How [`WebGpu::create_texture_from_url`] puts an image into a texture. By default, as it is:
/// the image's first row the texture's first, its colors not multiplied by its alpha, of one
/// level.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ImageOptions {
    /// With levels of smaller images of it, see [`WebGpu::generate_mipmaps`].
    pub mipmaps: bool,
    /// Its colors multiplied by its alpha, f. ex. for [`BlendState::PREMULTIPLIED_ALPHA_BLENDING`]:
    /// the smaller images of a see-through picture keep their edges then.
    pub premultiplied_alpha: bool,
    /// Its last row the texture's first.
    pub flip_y: bool,
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
    /// The layouts of its bind groups, or `None` for those WebGPU finds in its shaders, which
    /// serve this pipeline only, see [`RenderPipeline::get_bind_group_layout`].
    pub layout: Option<&'a PipelineLayout>,
    pub vertex: VertexState<'a>,
    pub primitive: PrimitiveState,
    /// Whether the pipeline tests and writes depth, of the pass's depth attachment.
    pub depth_stencil: Option<DepthStencilState>,
    /// What colors the fragments get, if any: a pipeline only writing depth needs none.
    pub fragment: Option<FragmentState<'a>>,
    /// Samples a pixel of the attachments it draws on.
    pub multisample: MultisampleState,
}

impl<'a> RenderPipelineDescriptor<'a> {
    /// A pipeline of `vertex` and `fragment`, with the layout WebGPU finds in its shaders, of
    /// triangles, without depth, of a sample a pixel.
    pub fn new(vertex: VertexState<'a>, fragment: Option<FragmentState<'a>>) -> Self {
        Self {
            layout: None,
            vertex,
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            fragment,
            multisample: MultisampleState::default(),
        }
    }

    fn put(&self, out: &mut Vec<u8>) {
        put_u32(out, self.layout.map_or(0, |layout| layout.0.id));
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

        out.push(self.multisample.count.max(1) as u8);
    }
}

/// See [`WebGpu::create_compute_pipeline`].
#[derive(Clone, Copy, Debug)]
pub struct ComputePipelineDescriptor<'a> {
    /// The layouts of its bind groups, or `None` for those WebGPU finds in its shader, which
    /// serve this pipeline only, see [`ComputePipeline::get_bind_group_layout`].
    pub layout: Option<&'a PipelineLayout>,
    pub module: &'a ShaderModule,
    pub entry_point: &'a str,
}

/// Samples a pixel a pipeline draws, see [`TextureDescriptor::sample_count`]: as many as the
/// attachments of the passes using it have.
///
/// ```rust
/// use vertigo::webgpu::MultisampleState;
///
/// // Smooth edges, for `Target::Multisampled(4)`
/// let smooth = MultisampleState { count: 4 };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MultisampleState {
    pub count: u32,
}

impl Default for MultisampleState {
    /// One sample a pixel.
    fn default() -> Self {
        Self { count: 1 }
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
            attachment.target.put(out);
            match attachment.resolve_target {
                Some(target) => {
                    out.push(1);
                    target.put(out);
                }
                None => out.push(0),
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
    /// Where the samples of a multisampled target go, a pixel each, when the pass ends: the canvas
    /// or a texture of 1 sample a pixel, of its format and size.
    pub resolve_target: Option<Target<'a>>,
    pub load: LoadOp<Color>,
    pub store: StoreOp,
}

impl ColorAttachment<'_> {
    /// The canvas's picture, kept.
    pub fn canvas(load: LoadOp<Color>) -> Self {
        Self {
            target: Target::Canvas,
            resolve_target: None,
            load,
            store: StoreOp::Store,
        }
    }

    /// The canvas's picture with `samples` samples a pixel, f. ex. 4, resolved into the canvas:
    /// its edges smooth. The pipelines of the pass need as many, see [`MultisampleState`], and so
    /// does a depth texture, which [`DepthTarget::Canvas`] has then.
    ///
    /// ```rust
    /// use vertigo::webgpu::{Color, ColorAttachment, LoadOp};
    ///
    /// let smooth = ColorAttachment::canvas_multisampled(4, LoadOp::Clear(Color::BLACK));
    /// ```
    pub fn canvas_multisampled(samples: u32, load: LoadOp<Color>) -> Self {
        Self {
            target: Target::Multisampled(samples),
            resolve_target: Some(Target::Canvas),
            load,
            store: StoreOp::Discard,
        }
    }
}

/// Where colors go.
#[derive(Clone, Copy, Debug)]
pub enum Target<'a> {
    /// The canvas's picture, in [`TextureFormat::Canvas`], shown once the batch is drawn.
    Canvas,
    Texture(&'a Texture),
    /// A texture of the canvas's format and size, of so many samples a pixel, made by the browser
    /// for the context and made again when the canvas changes its size: to be resolved, see
    /// [`ColorAttachment::canvas_multisampled`].
    Multisampled(u32),
}

impl Target<'_> {
    fn put(&self, out: &mut Vec<u8>) {
        match self {
            Target::Canvas => out.push(0),
            Target::Texture(texture) => {
                out.push(1);
                put_u32(out, texture.object.id);
            }
            Target::Multisampled(samples) => out.extend([2, (*samples).clamp(1, 255) as u8]),
        }
    }
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
    /// canvas. Of as many samples a pixel as the first color attachment of the pass has.
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
    /// Which shaders see a binding, joined with `|`, as WebGPU's `GPUShaderStage`.
    ShaderStages(u8) {
        VERTEX = 0x1,
        FRAGMENT = 0x2,
        COMPUTE = 0x4,
    }
);

/// A binding of a bind group layout, see [`WebGpu::create_bind_group_layout`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BindGroupLayoutEntry {
    /// Of `@binding(...)` in shaders.
    pub binding: u32,
    pub visibility: ShaderStages,
    pub ty: BindingType,
}

/// What a binding is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindingType {
    Buffer(BufferBindingType),
    Sampler(SamplerBindingType),
    /// A 2D texture, read through a sampler.
    Texture {
        sample_type: TextureSampleType,
        multisampled: bool,
    },
}

impl BindingType {
    /// Its kind on the wire, see the format, and whether it's multisampled.
    fn kind(self) -> (u8, bool) {
        match self {
            Self::Buffer(BufferBindingType::Uniform) => (0, false),
            Self::Buffer(BufferBindingType::Storage { read_only: false }) => (1, false),
            Self::Buffer(BufferBindingType::Storage { read_only: true }) => (2, false),
            Self::Sampler(SamplerBindingType::Filtering) => (3, false),
            Self::Sampler(SamplerBindingType::NonFiltering) => (4, false),
            Self::Sampler(SamplerBindingType::Comparison) => (5, false),
            Self::Texture {
                sample_type,
                multisampled,
            } => (
                match sample_type {
                    TextureSampleType::Float { filterable: true } => 6,
                    TextureSampleType::Float { filterable: false } => 7,
                    TextureSampleType::Depth => 8,
                    TextureSampleType::Sint => 9,
                    TextureSampleType::Uint => 10,
                },
                multisampled,
            ),
        }
    }
}

/// A buffer of a binding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BufferBindingType {
    /// `var<uniform>`
    Uniform,
    /// `var<storage, read_write>`, or `var<storage>` when `read_only`.
    Storage { read_only: bool },
}

/// A sampler of a binding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SamplerBindingType {
    /// Reading between pixels, of [`FilterMode::Linear`].
    Filtering,
    NonFiltering,
    Comparison,
}

/// What a texture of a binding gives a shader.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureSampleType {
    /// Floats, read between pixels if `filterable`: of colors, f. ex.
    Float {
        filterable: bool,
    },
    Depth,
    Sint,
    Uint,
}

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
            Self::Rg32Float | Self::Rgba16Float => 8,
            Self::Rgba32Float => 16,
            Self::Canvas
            | Self::Rg16Float
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
                bitmap_scale: 0.5,
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
            mip_level_count: 2,
            sample_count: 1,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        });
        gpu.write_texture(&texture, &[255, 0, 128, 255, 1, 2, 3, 4]);
        gpu.generate_mipmaps(&texture);
        let image = gpu.create_texture_from_url(
            "/a.png",
            &ImageOptions {
                mipmaps: true,
                premultiplied_alpha: true,
                flip_y: false,
            },
        );
        let sampler = gpu.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::MirrorRepeat,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Linear,
        });
        let shader = gpu.create_shader_module("wgsł");
        let layout = gpu.create_bind_group_layout(&[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer(BufferBindingType::Uniform),
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer(BufferBindingType::Storage { read_only: true }),
            },
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    multisampled: false,
                },
            },
            BindGroupLayoutEntry {
                binding: 3,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
            },
            BindGroupLayoutEntry {
                binding: 4,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Depth,
                    multisampled: true,
                },
            },
        ]);
        let pipeline_layout = gpu.create_pipeline_layout(&[&layout]);
        let pipeline = gpu.create_render_pipeline(&RenderPipelineDescriptor {
            layout: Some(&pipeline_layout),
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
            multisample: MultisampleState { count: 4 },
        });
        let depth_only = gpu.create_render_pipeline(&RenderPipelineDescriptor::new(
            VertexState {
                module: &shader,
                entry_point: "vs",
                buffers: &[],
            },
            None,
        ));
        let compute = gpu.create_compute_pipeline(&ComputePipelineDescriptor {
            layout: None,
            module: &shader,
            entry_point: "cs",
        });
        let group = gpu.create_bind_group(
            &layout,
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
                    resource: BindingResource::Texture(&image),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        );
        let computed = compute.get_bind_group_layout(0);
        let numbers = gpu.create_bind_group(
            &computed,
            &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(&buffer),
            }],
        );
        {
            let mut pass = gpu.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[
                    ColorAttachment::canvas_multisampled(
                        4,
                        LoadOp::Clear(Color {
                            r: 0.0,
                            g: 0.25,
                            b: 0.5,
                            a: 1.0,
                        }),
                    ),
                    ColorAttachment {
                        target: Target::Texture(&texture),
                        resolve_target: None,
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
            pass.set_bind_group(0, &numbers);
            pass.dispatch_workgroups(4, 2, 1);
        }
        gpu.read_buffer(&buffer, |_| {});
        gpu.read_pixels(Target::Canvas, |_, _| {});
        gpu.read_pixels(Target::Texture(&texture), |_, _| {});
        gpu.set_bitmap_scale(0.25);
        drop((
            buffer,
            texture,
            image,
            sampler,
            shader,
            layout,
            pipeline_layout,
            pipeline,
            depth_only,
            compute,
            group,
            computed,
            numbers,
        ));
        gpu.take()
    }

    /// The same bytes are played back in `driver_module/src_js/api/command/webgpu.test.ts`:
    /// change the format and one of the two tests fails, whichever side was changed.
    const FIXTURE: &[u8] = &[
        1, 1, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 63, 2, 1, 0, 0, 0, 8, 0, 0, 0, 40, 0, 0, 0, 4, 1,
        0, 0, 0, 4, 0, 0, 0, 8, 0, 0, 0, 0, 0, 192, 63, 0, 0, 0, 192, 4, 1, 0, 0, 0, 0, 0, 0, 0, 4,
        0, 0, 0, 7, 0, 0, 0, 5, 2, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 1, 6, 0, 0, 0, 2, 0, 0, 0, 1,
        7, 2, 0, 0, 0, 8, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 8, 0, 0, 0, 255, 0, 128, 255, 1, 2, 3,
        4, 34, 2, 0, 0, 0, 35, 3, 0, 0, 0, 1, 1, 0, 6, 0, 0, 0, 47, 97, 46, 112, 110, 103, 8, 4, 0,
        0, 0, 1, 2, 1, 0, 1, 10, 5, 0, 0, 0, 5, 0, 0, 0, 119, 103, 115, 197, 130, 29, 6, 0, 0, 0,
        5, 0, 0, 0, 0, 3, 0, 0, 1, 0, 0, 0, 4, 2, 0, 2, 0, 0, 0, 2, 6, 0, 3, 0, 0, 0, 2, 3, 0, 4,
        0, 0, 0, 2, 8, 1, 31, 7, 0, 0, 0, 1, 6, 0, 0, 0, 12, 8, 0, 0, 0, 7, 0, 0, 0, 5, 0, 0, 0, 2,
        0, 0, 0, 118, 115, 2, 24, 0, 0, 0, 0, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 12, 12, 0, 0, 0, 1, 0,
        0, 0, 64, 0, 0, 0, 1, 1, 3, 16, 0, 0, 0, 3, 0, 0, 0, 4, 2, 1, 2, 1, 15, 1, 3, 1, 5, 0, 0,
        0, 2, 0, 0, 0, 102, 115, 2, 0, 0, 15, 9, 1, 4, 5, 0, 1, 5, 0, 7, 4, 12, 9, 0, 0, 0, 0, 0,
        0, 0, 5, 0, 0, 0, 2, 0, 0, 0, 118, 115, 0, 3, 0, 0, 0, 0, 0, 1, 13, 10, 0, 0, 0, 0, 0, 0,
        0, 5, 0, 0, 0, 2, 0, 0, 0, 99, 115, 15, 11, 0, 0, 0, 6, 0, 0, 0, 4, 0, 0, 0, 0, 0, 1, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 64, 0, 0, 0, 2, 0, 0, 0,
        1, 3, 0, 0, 0, 3, 0, 0, 0, 2, 4, 0, 0, 0, 33, 12, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 15, 13,
        0, 0, 0, 12, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 17, 2, 2, 4, 1,
        0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 208, 63, 0, 0, 0, 0, 0, 0, 224, 63, 0, 0,
        0, 0, 0, 0, 240, 63, 1, 1, 2, 0, 0, 0, 0, 0, 1, 1, 0, 15, 1, 0, 0, 128, 63, 0, 20, 8, 0, 0,
        0, 21, 1, 0, 0, 0, 11, 0, 0, 0, 22, 1, 0, 0, 0, 1, 0, 0, 0, 8, 0, 0, 0, 23, 1, 0, 0, 0, 0,
        2, 0, 0, 0, 24, 3, 0, 0, 0, 10, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 25, 36, 0, 0, 0, 144, 1,
        0, 0, 6, 0, 0, 0, 255, 255, 255, 255, 2, 0, 0, 0, 27, 0, 0, 0, 0, 0, 0, 128, 63, 0, 0, 32,
        68, 0, 0, 240, 67, 0, 0, 0, 0, 0, 0, 128, 63, 28, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0,
        0, 0, 19, 17, 0, 1, 1, 2, 0, 0, 0, 0, 1, 19, 18, 20, 10, 0, 0, 0, 21, 0, 0, 0, 0, 13, 0, 0,
        0, 26, 4, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 19, 36, 1, 0, 0, 0, 1, 0, 0, 0, 37, 2, 0, 0, 0,
        0, 37, 3, 0, 0, 0, 1, 2, 0, 0, 0, 38, 0, 0, 128, 62, 3, 1, 0, 0, 0, 6, 2, 0, 0, 0, 6, 3, 0,
        0, 0, 9, 4, 0, 0, 0, 11, 5, 0, 0, 0, 30, 6, 0, 0, 0, 32, 7, 0, 0, 0, 14, 8, 0, 0, 0, 14, 9,
        0, 0, 0, 14, 10, 0, 0, 0, 16, 11, 0, 0, 0, 30, 12, 0, 0, 0, 16, 13, 0, 0, 0,
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
        // The context's callback, the first of the test's thread, and the scale of its bitmap
        calls.extend(1_u64.to_le_bytes());
        calls.extend(1.0_f32.to_le_bytes());
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
        let tell = json_teller(gpu);
        move |values: &[u32]| tell(values.iter().copied().map(number).collect())
    }

    /// Tells `gpu`'s context a list of `values` of any kind, as the browser does.
    fn json_teller(gpu: &WebGpu) -> impl Fn(Vec<JsJson>) {
        let calls = gpu.take();
        let Ok(callback) = <[u8; 8]>::try_from(&calls[3..11]) else {
            panic!("a context starts with its callback: {calls:?}");
        };
        let callback = CallbackId::from_u64(u64::from_le_bytes(callback));
        move |values: Vec<JsJson>| {
            api_callbacks().call(callback, JsJson::List(values));
        }
    }

    fn number(value: u32) -> JsJson {
        JsJson::Number(JsJsonNumber(f64::from(value)))
    }

    #[test]
    fn an_image_that_came_is_told_and_gives_its_texture_its_size() {
        let canvas = dom_element! { <canvas /> };
        let gpu = WebGpu::new(&canvas.get_ref());
        let tell = teller(&gpu);
        let loads = Rc::new(RefCell::new(Vec::new()));
        let _loaded = gpu.on_image_load({
            let loads = loads.clone();
            move |load| loads.borrow_mut().push(load.clone())
        });

        let bricks = gpu.create_texture_from_url("/bricks.png", &ImageOptions::default());
        let stones = gpu.create_texture_from_url("/stones.png", &ImageOptions::default());
        let one = Size {
            width: 1,
            height: 1,
        };
        assert_eq!(bricks.size(), one, "a pixel until the image comes");

        tell(&[3, bricks.object.id, 64, 32]);
        tell(&[4, stones.object.id]);
        tell(&[3, bricks.object.id, 64, 32]);
        let size = Size {
            width: 64,
            height: 32,
        };
        assert_eq!(bricks.size(), size);
        assert_eq!(stones.size(), one);
        assert_eq!(
            *loads.borrow(),
            vec![
                ImageLoad::Loaded {
                    url: "/bricks.png".to_string(),
                    size,
                },
                ImageLoad::Failed {
                    url: "/stones.png".to_string(),
                },
            ],
            "each told once"
        );
    }

    #[test]
    fn a_read_is_answered_once_and_not_after_the_device_is_lost() {
        let canvas = dom_element! { <canvas /> };
        let gpu = WebGpu::new(&canvas.get_ref());
        let tell = json_teller(&gpu);
        let buffer = gpu.create_buffer(4, BufferUsages::COPY_SRC);
        let read = Rc::new(RefCell::new(Vec::new()));
        let answer = |read: &Rc<RefCell<Vec<String>>>| {
            let read = read.clone();
            move |bytes: Vec<u8>| read.borrow_mut().push(format!("{bytes:?}"))
        };

        gpu.read_buffer(&buffer, answer(&read));
        gpu.read_pixels(Target::Canvas, {
            let read = read.clone();
            move |size, pixels| {
                read.borrow_mut()
                    .push(format!("{}x{} {pixels:?}", size.width, size.height))
            }
        });
        tell(vec![
            number(6),
            number(2),
            number(1),
            number(1),
            JsJson::Vec(vec![1, 2, 3, 4]),
        ]);
        tell(vec![number(5), number(1), JsJson::Vec(vec![7, 0, 0, 0])]);
        tell(vec![number(5), number(1), JsJson::Vec(vec![8, 0, 0, 0])]);
        assert_eq!(*read.borrow(), vec!["1x1 [1, 2, 3, 4]", "[7, 0, 0, 0]"]);

        gpu.read_buffer(&buffer, answer(&read));
        tell(vec![number(1)]);
        tell(vec![number(5), number(3), JsJson::Vec(vec![9, 0, 0, 0])]);
        assert_eq!(read.borrow().len(), 2, "gone with the device");
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
    fn a_pixel_is_as_many_bytes_as_its_channels_take() {
        assert_eq!(TextureFormat::R8Unorm.bytes_per_pixel(), 1);
        assert_eq!(TextureFormat::Rg16Float.bytes_per_pixel(), 4);
        assert_eq!(TextureFormat::Rgba16Float.bytes_per_pixel(), 8);
        assert_eq!(TextureFormat::Rgba32Float.bytes_per_pixel(), 16);
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
