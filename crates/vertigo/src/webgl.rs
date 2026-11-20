//! WebGL 2 on a `<canvas>`, see [`WebGl`].
//!
//! ## Format
//!
//! The calls go to the browser as commands, which it reads in
//! `driver_module/src_js/api/command/webgl.ts`, and `webgl.test.ts` there checks it against bytes
//! made here. Numbers are little-endian.
//!
//! ```text
//! commands := op*
//! op       := code:u8 args               -- see `code`
//! id       := u32                        -- of a buffer, a vertex array, a program, a texture or
//!                                           a uniform's location, 0 for none
//! name     := length:u16 utf8*           -- of a uniform, to find its location
//! source   := length:u32 utf8*           -- of a shader
//! data     := length:u32 byte*
//! ```
//!
//! A WebGL constant, f. ex. of [`Primitive`], travels as its number, a `u32`, so the browser
//! needs no table of them.

use std::{
    cell::Cell,
    fmt,
    rc::{Rc, Weak},
};

use crate::{DomElementRef, driver_module::get_driver_dom, struct_mut::ValueMut};

/// Operation codes. Append only, as the browser reads them by number.
mod code {
    /// The context's attributes, see `WebGlOptions`: always the first command of a `WebGl`.
    pub const CONTEXT: u8 = 1;
    pub const CREATE_BUFFER: u8 = 2;
    pub const DELETE_BUFFER: u8 = 3;
    pub const BIND_BUFFER: u8 = 4;
    pub const BUFFER_DATA: u8 = 5;
    pub const BUFFER_SUB_DATA: u8 = 6;
    pub const CREATE_VERTEX_ARRAY: u8 = 7;
    pub const DELETE_VERTEX_ARRAY: u8 = 8;
    pub const BIND_VERTEX_ARRAY: u8 = 9;
    pub const ENABLE_VERTEX_ATTRIB_ARRAY: u8 = 10;
    pub const VERTEX_ATTRIB_POINTER: u8 = 11;
    pub const CREATE_PROGRAM: u8 = 12;
    pub const DELETE_PROGRAM: u8 = 13;
    pub const USE_PROGRAM: u8 = 14;
    pub const UNIFORM_1F: u8 = 15;
    pub const UNIFORM_2F: u8 = 16;
    pub const UNIFORM_3F: u8 = 17;
    pub const UNIFORM_4F: u8 = 18;
    pub const UNIFORM_1I: u8 = 19;
    pub const UNIFORM_MATRIX_3F: u8 = 20;
    pub const UNIFORM_MATRIX_4F: u8 = 21;
    pub const CREATE_TEXTURE: u8 = 22;
    pub const DELETE_TEXTURE: u8 = 23;
    pub const ACTIVE_TEXTURE: u8 = 24;
    pub const BIND_TEXTURE: u8 = 25;
    pub const TEXTURE_IMAGE: u8 = 26;
    pub const TEXTURE_PARAMETER: u8 = 27;
    pub const GENERATE_MIPMAP: u8 = 28;
    pub const CLEAR_COLOR: u8 = 29;
    pub const CLEAR_DEPTH: u8 = 30;
    pub const CLEAR: u8 = 31;
    pub const ENABLE: u8 = 32;
    pub const DISABLE: u8 = 33;
    pub const VIEWPORT: u8 = 34;
    pub const DEPTH_FUNC: u8 = 35;
    pub const BLEND_FUNC: u8 = 36;
    pub const CULL_FACE: u8 = 37;
    pub const FRONT_FACE: u8 = 38;
    pub const DRAW_ARRAYS: u8 = 39;
    pub const DRAW_ELEMENTS: u8 = 40;
    pub const UNIFORM_LOCATION: u8 = 41;
    pub const DELETE_UNIFORM_LOCATION: u8 = 42;
}

/// `pname` of `texParameteri`.
const TEXTURE_MAG_FILTER: u32 = 0x2800;
const TEXTURE_MIN_FILTER: u32 = 0x2801;
const TEXTURE_WRAP_S: u32 = 0x2802;
const TEXTURE_WRAP_T: u32 = 0x2803;

/// The commands of a context not yet sent, shared by it and its objects. The browser gets them at
/// the end of the transaction, see `DriverDom::flush_dom_changes`.
pub(crate) type Queue = ValueMut<Vec<u8>>;

thread_local! {
    /// Ids of objects, unique among all contexts, so two `WebGl`s of one canvas don't mix them up.
    static LAST_ID: Cell<u32> = const { Cell::new(0) };
}

/// The attributes a canvas gets its WebGL 2 context with, as `getContext("webgl2", ...)` takes
/// them. The first [`WebGl`] of a canvas sets them: the browser keeps the context it made.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WebGlOptions {
    /// Whether the canvas is see-through where the drawing is.
    pub alpha: bool,
    pub antialias: bool,
    /// Whether there is a depth buffer, for [`Capability::DepthTest`].
    pub depth: bool,
    pub stencil: bool,
    pub premultiplied_alpha: bool,
    /// Whether what was drawn stays after the browser shows it, until it is cleared, instead of
    /// being cleared then. Slower, but the pixels can be read back, f. ex. in a test.
    pub preserve_drawing_buffer: bool,
}

impl Default for WebGlOptions {
    /// As a browser has them.
    fn default() -> Self {
        Self {
            alpha: true,
            antialias: true,
            depth: true,
            stencil: false,
            premultiplied_alpha: true,
            preserve_drawing_buffer: false,
        }
    }
}

impl WebGlOptions {
    fn flags(&self) -> u8 {
        [
            self.alpha,
            self.antialias,
            self.depth,
            self.stencil,
            self.premultiplied_alpha,
            self.preserve_drawing_buffer,
        ]
        .iter()
        .enumerate()
        .fold(0, |flags, (bit, on)| flags | (u8::from(*on) << bit))
    }
}

/// WebGL 2 on a `<canvas>`: its calls, with handles in place of WebGL's objects, which are let go
/// when they are dropped.
///
/// The calls of a transaction go to the browser together at its end, in the same batch as the
/// changes to the page, after the canvas is on it. So a frame is drawn by calls made in
/// [`Driver::on_animation_frame`](crate::Driver::on_animation_frame), and the objects it uses are
/// made once, beforehand. Nothing is drawn during SSR.
///
/// No call waits for the browser to answer. [`Self::uniform_location`] gives a handle at once,
/// which the browser finds the location for when the call comes, and an attribute has the
/// location its shader gives it (`layout(location = 0)`). A shader that doesn't compile, or a
/// program that doesn't link, is reported in the browser's console.
///
/// The bitmap of the canvas is as big as the canvas is on the screen, in device pixels, so it is
/// sharp. When that size changes, the viewport becomes the whole canvas again.
///
/// ```rust
/// use vertigo::{DomNode, dom_element, get_driver};
/// use vertigo::webgl::{
///     BufferTarget, BufferUsage, ClearBuffers, DataType, Primitive, WebGl,
/// };
///
/// // A triangle, its color going round.
/// fn triangle() -> DomNode {
///     let canvas = dom_element! { <canvas style="width: 200px; height: 200px" /> };
///     let gl = WebGl::new(&canvas.get_ref());
///
///     let program = gl.create_program(
///         "#version 300 es
///         layout(location = 0) in vec2 position;
///         void main() { gl_Position = vec4(position, 0.0, 1.0); }",
///         "#version 300 es
///         precision mediump float;
///         uniform vec3 color;
///         out vec4 pixel;
///         void main() { pixel = vec4(color, 1.0); }",
///     );
///     let color = gl.uniform_location(&program, "color");
///
///     let vertices = gl.create_vertex_array();
///     gl.bind_vertex_array(Some(&vertices));
///     let buffer = gl.create_buffer();
///     gl.bind_buffer(BufferTarget::Array, Some(&buffer));
///     gl.buffer_data(BufferTarget::Array, &[0.0, 0.8, -0.8, -0.8, 0.8, -0.8_f32], BufferUsage::StaticDraw);
///     gl.enable_vertex_attrib_array(0);
///     gl.vertex_attrib_pointer(0, 2, DataType::Float, false, 0, 0);
///
///     canvas.append_drop_resource(get_driver().on_animation_frame(move |time| {
///         let turn = (time / 1000.0) as f32;
///         gl.clear_color(0.0, 0.0, 0.0, 1.0);
///         gl.clear(ClearBuffers::COLOR);
///         gl.use_program(Some(&program));
///         gl.uniform_3f(&color, turn.sin().abs(), turn.cos().abs(), 0.5);
///         gl.bind_vertex_array(Some(&vertices));
///         gl.draw_arrays(Primitive::Triangles, 0, 3);
///         // The buffer lives as long as the frames do.
///         let _ = &buffer;
///     }));
///
///     canvas.into()
/// }
/// ```
#[derive(Clone)]
pub struct WebGl {
    queue: Rc<Queue>,
}

impl fmt::Debug for WebGl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebGl").finish_non_exhaustive()
    }
}

impl WebGl {
    /// WebGL 2 on `canvas`, with the attributes a browser gives a context by default.
    pub fn new(canvas: &DomElementRef) -> Self {
        Self::with_options(canvas, WebGlOptions::default())
    }

    /// WebGL 2 on `canvas`, with a context of these attributes.
    ///
    /// ```rust
    /// use vertigo::dom_element;
    /// use vertigo::webgl::{WebGl, WebGlOptions};
    ///
    /// let canvas = dom_element! { <canvas /> };
    /// let gl = WebGl::with_options(&canvas.get_ref(), WebGlOptions {
    ///     preserve_drawing_buffer: true,
    ///     ..WebGlOptions::default()
    /// });
    /// ```
    pub fn with_options(canvas: &DomElementRef, options: WebGlOptions) -> Self {
        let queue = Rc::new(ValueMut::new(vec![code::CONTEXT, options.flags()]));
        get_driver_dom().webgl_queue(canvas.id(), Rc::downgrade(&queue));
        Self { queue }
    }

    fn write(&self, write: impl FnOnce(&mut Vec<u8>)) {
        self.queue.change(write);
    }

    fn op(&self, code: u8, write: impl FnOnce(&mut Vec<u8>)) {
        self.write(|out| {
            out.push(code);
            write(out);
        });
    }

    /// An object made with `create`, let go with `delete` when its handle is dropped.
    fn object(&self, create: u8, delete: u8) -> Object {
        let id = LAST_ID.with(|last| {
            last.set(last.get() + 1);
            last.get()
        });
        self.op(create, |out| put_u32(out, id));
        Object {
            id,
            delete,
            queue: Rc::downgrade(&self.queue),
        }
    }

    pub fn create_buffer(&self) -> Buffer {
        Buffer(self.object(code::CREATE_BUFFER, code::DELETE_BUFFER))
    }

    pub fn bind_buffer(&self, target: BufferTarget, buffer: Option<&Buffer>) {
        self.op(code::BIND_BUFFER, |out| {
            put_u32(out, target as u32);
            put_u32(out, id_of(buffer.map(|buffer| &buffer.0)));
        });
    }

    /// Fills the buffer bound to `target` with `data`, as big as it is.
    pub fn buffer_data<T: Scalar>(&self, target: BufferTarget, data: &[T], usage: BufferUsage) {
        self.op(code::BUFFER_DATA, |out| {
            put_u32(out, target as u32);
            put_u32(out, usage as u32);
            put_data(out, data);
        });
    }

    /// Writes `data` into the buffer bound to `target`, from `offset` bytes into it.
    pub fn buffer_sub_data<T: Scalar>(&self, target: BufferTarget, offset: u32, data: &[T]) {
        self.op(code::BUFFER_SUB_DATA, |out| {
            put_u32(out, target as u32);
            put_u32(out, offset);
            put_data(out, data);
        });
    }

    pub fn create_vertex_array(&self) -> VertexArray {
        VertexArray(self.object(code::CREATE_VERTEX_ARRAY, code::DELETE_VERTEX_ARRAY))
    }

    pub fn bind_vertex_array(&self, vertex_array: Option<&VertexArray>) {
        self.op(code::BIND_VERTEX_ARRAY, |out| {
            put_u32(out, id_of(vertex_array.map(|array| &array.0)));
        });
    }

    pub fn enable_vertex_attrib_array(&self, index: u32) {
        self.op(code::ENABLE_VERTEX_ATTRIB_ARRAY, |out| put_u32(out, index));
    }

    /// Where the attribute at `index` takes its values from: the buffer bound to
    /// [`BufferTarget::Array`], `size` values (1 to 4) of `kind` a vertex, a vertex every `stride`
    /// bytes (0 for right after one another), from `offset` bytes into it.
    pub fn vertex_attrib_pointer(
        &self,
        index: u32,
        size: u8,
        kind: DataType,
        normalized: bool,
        stride: u32,
        offset: u32,
    ) {
        self.op(code::VERTEX_ATTRIB_POINTER, |out| {
            put_u32(out, index);
            out.push(size);
            put_u32(out, kind as u32);
            out.push(u8::from(normalized));
            put_u32(out, stride);
            put_u32(out, offset);
        });
    }

    /// A program of a vertex and a fragment shader in GLSL ES, compiled and linked. If either
    /// doesn't compile or they don't link, the browser's console says why, and drawing with the
    /// program draws nothing.
    pub fn create_program(&self, vertex: &str, fragment: &str) -> Program {
        let program = Program(self.object(code::CREATE_PROGRAM, code::DELETE_PROGRAM));
        self.write(|out| {
            put_source(out, vertex);
            put_source(out, fragment);
        });
        program
    }

    /// The program the next calls draw with, and set the uniforms of.
    pub fn use_program(&self, program: Option<&Program>) {
        self.op(code::USE_PROGRAM, |out| {
            put_u32(out, id_of(program.map(|program| &program.0)));
        });
    }

    /// Where the uniform `name` of `program` is, to set it with, as `getUniformLocation` finds
    /// it. Found once, beforehand, for the calls setting it to carry a number rather than the
    /// name. A program without the uniform gets a location setting nothing.
    ///
    /// ```rust
    /// use vertigo::dom_element;
    /// use vertigo::webgl::WebGl;
    ///
    /// # let canvas = dom_element! { <canvas /> };
    /// # let gl = WebGl::new(&canvas.get_ref());
    /// # let program = gl.create_program("", "");
    /// let transform = gl.uniform_location(&program, "transform");
    /// // Then, for each frame:
    /// gl.use_program(Some(&program));
    /// gl.uniform_matrix_4f(&transform, &[1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]);
    /// ```
    pub fn uniform_location(&self, program: &Program, name: &str) -> UniformLocation {
        let location =
            UniformLocation(self.object(code::UNIFORM_LOCATION, code::DELETE_UNIFORM_LOCATION));
        self.write(|out| {
            put_u32(out, program.0.id);
            put_name(out, name);
        });
        location
    }

    fn uniform(&self, code: u8, location: &UniformLocation, values: &[f32]) {
        self.op(code, |out| {
            put_u32(out, location.0.id);
            put_f32s(out, values);
        });
    }

    /// Sets a uniform of the program in use, which `location` has to be of.
    pub fn uniform_1f(&self, location: &UniformLocation, x: f32) {
        self.uniform(code::UNIFORM_1F, location, &[x]);
    }

    pub fn uniform_2f(&self, location: &UniformLocation, x: f32, y: f32) {
        self.uniform(code::UNIFORM_2F, location, &[x, y]);
    }

    pub fn uniform_3f(&self, location: &UniformLocation, x: f32, y: f32, z: f32) {
        self.uniform(code::UNIFORM_3F, location, &[x, y, z]);
    }

    pub fn uniform_4f(&self, location: &UniformLocation, x: f32, y: f32, z: f32, w: f32) {
        self.uniform(code::UNIFORM_4F, location, &[x, y, z, w]);
    }

    /// Sets an `int` or a `bool` uniform, or a sampler to the texture unit it reads.
    pub fn uniform_1i(&self, location: &UniformLocation, x: i32) {
        self.op(code::UNIFORM_1I, |out| {
            put_u32(out, location.0.id);
            out.extend_from_slice(&x.to_le_bytes());
        });
    }

    /// Sets a `mat3` uniform, column after column.
    pub fn uniform_matrix_3f(&self, location: &UniformLocation, matrix: &[f32; 9]) {
        self.uniform(code::UNIFORM_MATRIX_3F, location, matrix);
    }

    /// Sets a `mat4` uniform, column after column.
    pub fn uniform_matrix_4f(&self, location: &UniformLocation, matrix: &[f32; 16]) {
        self.uniform(code::UNIFORM_MATRIX_4F, location, matrix);
    }

    pub fn create_texture(&self) -> Texture {
        Texture(self.object(code::CREATE_TEXTURE, code::DELETE_TEXTURE))
    }

    /// The texture unit, from 0, that the next texture calls are about.
    pub fn active_texture(&self, unit: u32) {
        self.op(code::ACTIVE_TEXTURE, |out| put_u32(out, unit));
    }

    /// The 2D texture of the active unit, that the next texture calls are about.
    pub fn bind_texture(&self, texture: Option<&Texture>) {
        self.op(code::BIND_TEXTURE, |out| {
            put_u32(out, id_of(texture.map(|texture| &texture.0)));
        });
    }

    /// The image of the bound texture: `pixels`, rows `width` wide of 4 bytes each, red, green,
    /// blue and alpha, from the top.
    pub fn texture_image(&self, width: u32, height: u32, pixels: &[u8]) {
        self.op(code::TEXTURE_IMAGE, |out| {
            put_u32(out, width);
            put_u32(out, height);
            put_data(out, pixels);
        });
    }

    /// How the bound texture is read between its pixels: when it is shown smaller (`min`) and
    /// bigger (`mag`) than it is. A filter with mipmaps needs [`Self::generate_mipmap`].
    pub fn texture_filter(&self, min: TextureFilter, mag: TextureFilter) {
        self.texture_parameter(TEXTURE_MIN_FILTER, min as u32);
        self.texture_parameter(TEXTURE_MAG_FILTER, mag as u32);
    }

    /// How the bound texture is read outside of it, across (`s`) and down (`t`).
    pub fn texture_wrap(&self, s: TextureWrap, t: TextureWrap) {
        self.texture_parameter(TEXTURE_WRAP_S, s as u32);
        self.texture_parameter(TEXTURE_WRAP_T, t as u32);
    }

    fn texture_parameter(&self, name: u32, value: u32) {
        self.op(code::TEXTURE_PARAMETER, |out| {
            put_u32(out, name);
            put_u32(out, value);
        });
    }

    /// Smaller images of the bound texture, for its filters with mipmaps.
    pub fn generate_mipmap(&self) {
        self.op(code::GENERATE_MIPMAP, |_| {});
    }

    /// The color [`Self::clear`] fills with.
    pub fn clear_color(&self, red: f32, green: f32, blue: f32, alpha: f32) {
        self.op(code::CLEAR_COLOR, |out| {
            put_f32s(out, &[red, green, blue, alpha]);
        });
    }

    /// The depth [`Self::clear`] fills with, 1 at first: the farthest.
    pub fn clear_depth(&self, depth: f32) {
        self.op(code::CLEAR_DEPTH, |out| put_f32s(out, &[depth]));
    }

    pub fn clear(&self, buffers: ClearBuffers) {
        self.op(code::CLEAR, |out| put_u32(out, buffers.0));
    }

    pub fn enable(&self, capability: Capability) {
        self.op(code::ENABLE, |out| put_u32(out, capability as u32));
    }

    pub fn disable(&self, capability: Capability) {
        self.op(code::DISABLE, |out| put_u32(out, capability as u32));
    }

    /// The part of the canvas drawn on, in device pixels from its bottom left corner. The whole
    /// canvas at first, and again whenever its size changes.
    pub fn viewport(&self, x: i32, y: i32, width: i32, height: i32) {
        self.op(code::VIEWPORT, |out| {
            for value in [x, y, width, height] {
                out.extend_from_slice(&value.to_le_bytes());
            }
        });
    }

    /// Which fragments [`Capability::DepthTest`] lets through, by their depth against what is
    /// there: [`Compare::Less`] at first.
    pub fn depth_func(&self, compare: Compare) {
        self.op(code::DEPTH_FUNC, |out| put_u32(out, compare as u32));
    }

    /// How [`Capability::Blend`] blends: the color drawn times `source`, plus what is there times
    /// `destination`.
    pub fn blend_func(&self, source: BlendFactor, destination: BlendFactor) {
        self.op(code::BLEND_FUNC, |out| {
            put_u32(out, source as u32);
            put_u32(out, destination as u32);
        });
    }

    /// Which faces [`Capability::CullFace`] leaves out: [`Face::Back`] at first.
    pub fn cull_face(&self, face: Face) {
        self.op(code::CULL_FACE, |out| put_u32(out, face as u32));
    }

    /// Which way round the corners of a front face go: [`Winding::CounterClockwise`] at first.
    pub fn front_face(&self, winding: Winding) {
        self.op(code::FRONT_FACE, |out| put_u32(out, winding as u32));
    }

    /// Draws `count` vertices of the bound vertex array, from the `first`.
    pub fn draw_arrays(&self, mode: Primitive, first: u32, count: u32) {
        self.op(code::DRAW_ARRAYS, |out| {
            put_u32(out, mode as u32);
            put_u32(out, first);
            put_u32(out, count);
        });
    }

    /// Draws `count` vertices of the bound vertex array, at the indices in the buffer bound to
    /// [`BufferTarget::ElementArray`], from `offset` bytes into it.
    pub fn draw_elements(&self, mode: Primitive, count: u32, kind: IndexType, offset: u32) {
        self.op(code::DRAW_ELEMENTS, |out| {
            put_u32(out, mode as u32);
            put_u32(out, count);
            put_u32(out, kind as u32);
            put_u32(out, offset);
        });
    }
}

/// A WebGL object, let go when dropped. Its context may be gone by then, and with it the object.
struct Object {
    id: u32,
    delete: u8,
    queue: Weak<Queue>,
}

impl Drop for Object {
    fn drop(&mut self) {
        if let Some(queue) = self.queue.upgrade() {
            queue.change(|out| {
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

fn id_of(object: Option<&Object>) -> u32 {
    object.map_or(0, |object| object.id)
}

/// A buffer of vertices or indices, see [`WebGl::create_buffer`].
#[derive(Debug)]
pub struct Buffer(Object);

/// Which attributes take their values from which buffers, see [`WebGl::create_vertex_array`].
#[derive(Debug)]
pub struct VertexArray(Object);

/// A vertex and a fragment shader linked, see [`WebGl::create_program`].
#[derive(Debug)]
pub struct Program(Object);

/// A 2D texture, see [`WebGl::create_texture`].
#[derive(Debug)]
pub struct Texture(Object);

/// Where a uniform of a program is, see [`WebGl::uniform_location`].
#[derive(Debug)]
pub struct UniformLocation(Object);

mod sealed {
    pub trait Sealed {}
}

/// A number a buffer holds: [`WebGl::buffer_data`] takes a slice of any of them.
pub trait Scalar: sealed::Sealed + Copy {
    #[doc(hidden)]
    fn put(self, out: &mut Vec<u8>);
}

macro_rules! scalars {
    ($($kind:ty),*) => {
        $(
            impl sealed::Sealed for $kind {}

            impl Scalar for $kind {
                fn put(self, out: &mut Vec<u8>) {
                    out.extend_from_slice(&self.to_le_bytes());
                }
            }
        )*
    };
}

scalars!(u8, i8, u16, i16, u32, i32, f32);

/// Where a buffer is bound, as WebGL's `target`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum BufferTarget {
    /// Of vertex attributes.
    Array = 0x8892,
    /// Of indices, for [`WebGl::draw_elements`].
    ElementArray = 0x8893,
}

/// How a buffer's data is used, which tells the browser where to keep it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum BufferUsage {
    /// Set once, drawn many times.
    StaticDraw = 0x88E4,
    /// Set again and again, drawn many times.
    DynamicDraw = 0x88E8,
    /// Set once, drawn a few times.
    StreamDraw = 0x88E0,
}

/// The kind of the values of an attribute in a buffer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum DataType {
    Byte = 0x1400,
    UnsignedByte = 0x1401,
    Short = 0x1402,
    UnsignedShort = 0x1403,
    Int = 0x1404,
    UnsignedInt = 0x1405,
    Float = 0x1406,
    HalfFloat = 0x140B,
}

/// The kind of the indices in a buffer, see [`WebGl::draw_elements`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum IndexType {
    UnsignedByte = 0x1401,
    UnsignedShort = 0x1403,
    UnsignedInt = 0x1405,
}

/// What vertices make, as WebGL's `mode`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Primitive {
    Points = 0x0000,
    Lines = 0x0001,
    LineLoop = 0x0002,
    LineStrip = 0x0003,
    Triangles = 0x0004,
    TriangleStrip = 0x0005,
    TriangleFan = 0x0006,
}

/// What [`WebGl::enable`] turns on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Capability {
    /// Blending, see [`WebGl::blend_func`].
    Blend = 0x0BE2,
    /// Leaving out faces, see [`WebGl::cull_face`].
    CullFace = 0x0B44,
    /// Leaving out what is behind what was drawn, see [`WebGl::depth_func`].
    DepthTest = 0x0B71,
    PolygonOffsetFill = 0x8037,
    RasterizerDiscard = 0x8C89,
    SampleAlphaToCoverage = 0x809E,
    SampleCoverage = 0x80A0,
    ScissorTest = 0x0C11,
    StencilTest = 0x0B90,
}

/// The buffers [`WebGl::clear`] clears, joined with `|`.
///
/// ```rust
/// use vertigo::webgl::ClearBuffers;
///
/// let both = ClearBuffers::COLOR | ClearBuffers::DEPTH;
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClearBuffers(u32);

impl ClearBuffers {
    pub const COLOR: Self = Self(0x4000);
    pub const DEPTH: Self = Self(0x0100);
    pub const STENCIL: Self = Self(0x0400);
}

impl std::ops::BitOr for ClearBuffers {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

/// How a value is compared with what is there, f. ex. the depth of a fragment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Compare {
    Never = 0x0200,
    Less = 0x0201,
    Equal = 0x0202,
    LessOrEqual = 0x0203,
    Greater = 0x0204,
    NotEqual = 0x0205,
    GreaterOrEqual = 0x0206,
    Always = 0x0207,
}

/// What a color is multiplied by when it is blended, see [`WebGl::blend_func`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum BlendFactor {
    Zero = 0,
    One = 1,
    SourceColor = 0x0300,
    OneMinusSourceColor = 0x0301,
    SourceAlpha = 0x0302,
    OneMinusSourceAlpha = 0x0303,
    DestinationAlpha = 0x0304,
    OneMinusDestinationAlpha = 0x0305,
    DestinationColor = 0x0306,
    OneMinusDestinationColor = 0x0307,
    SourceAlphaSaturate = 0x0308,
}

/// Faces of triangles, see [`WebGl::cull_face`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Face {
    Front = 0x0404,
    Back = 0x0405,
    FrontAndBack = 0x0408,
}

/// Which way round the corners of a triangle go, as the screen shows them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Winding {
    Clockwise = 0x0900,
    CounterClockwise = 0x0901,
}

/// How a texture is read between its pixels, see [`WebGl::texture_filter`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum TextureFilter {
    /// The nearest pixel: sharp, f. ex. for pixel art.
    Nearest = 0x2600,
    /// Between the four nearest pixels.
    Linear = 0x2601,
    NearestMipmapNearest = 0x2700,
    LinearMipmapNearest = 0x2701,
    NearestMipmapLinear = 0x2702,
    /// Between the four nearest pixels of the two nearest mipmaps: the smoothest.
    LinearMipmapLinear = 0x2703,
}

/// How a texture is read outside of it, see [`WebGl::texture_wrap`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum TextureWrap {
    Repeat = 0x2901,
    ClampToEdge = 0x812F,
    MirroredRepeat = 0x8370,
}

fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_f32s(out: &mut Vec<u8>, values: &[f32]) {
    for value in values {
        out.extend_from_slice(&value.to_le_bytes());
    }
}

fn put_data<T: Scalar>(out: &mut Vec<u8>, data: &[T]) {
    put_u32(out, (std::mem::size_of_val(data)) as u32);
    for value in data {
        value.put(out);
    }
}

/// A name longer than 65535 bytes is cut there: no uniform has such.
fn put_name(out: &mut Vec<u8>, name: &str) {
    let length = name.len().min(usize::from(u16::MAX));
    out.extend_from_slice(&(length as u16).to_le_bytes());
    out.extend_from_slice(&name.as_bytes()[..length]);
}

fn put_source(out: &mut Vec<u8>, source: &str) {
    put_u32(out, source.len() as u32);
    out.extend_from_slice(source.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        dev::{
            command::DriverDomCommand,
            inspect::{log_start, log_take},
        },
        dom_element, transaction,
    };

    impl WebGl {
        /// The calls not sent yet, taken.
        fn take(&self) -> Vec<u8> {
            self.queue.change(std::mem::take)
        }
    }

    /// One call of each. A test runs on a thread of its own, so the objects' ids are 1 to 5, in
    /// the order they are made.
    fn fixture() -> Vec<u8> {
        let canvas = dom_element! { <canvas /> };
        let gl = WebGl::with_options(
            &canvas.get_ref(),
            WebGlOptions {
                stencil: true,
                preserve_drawing_buffer: true,
                ..WebGlOptions::default()
            },
        );
        let buffer = gl.create_buffer();
        gl.bind_buffer(BufferTarget::Array, Some(&buffer));
        gl.buffer_data(
            BufferTarget::Array,
            &[1.5_f32, -2.0],
            BufferUsage::StaticDraw,
        );
        gl.buffer_sub_data(BufferTarget::ElementArray, 2, &[7_u16, 8]);
        let vertices = gl.create_vertex_array();
        gl.bind_vertex_array(Some(&vertices));
        gl.enable_vertex_attrib_array(1);
        gl.vertex_attrib_pointer(1, 3, DataType::Float, true, 12, 4);
        let program = gl.create_program("vs", "fś");
        gl.use_program(Some(&program));
        let location = gl.uniform_location(&program, "u");
        gl.uniform_1f(&location, 0.5);
        gl.uniform_2f(&location, 1.0, 2.0);
        gl.uniform_3f(&location, 1.0, 2.0, 3.0);
        gl.uniform_4f(&location, 1.0, 2.0, 3.0, 4.0);
        gl.uniform_1i(&location, -1);
        gl.uniform_matrix_3f(&location, &[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
        gl.uniform_matrix_4f(
            &location,
            &[
                0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0,
                15.0,
            ],
        );
        let texture = gl.create_texture();
        gl.active_texture(2);
        gl.bind_texture(Some(&texture));
        gl.texture_image(1, 1, &[255, 0, 128, 255]);
        gl.texture_filter(TextureFilter::Nearest, TextureFilter::Linear);
        gl.texture_wrap(TextureWrap::Repeat, TextureWrap::ClampToEdge);
        gl.generate_mipmap();
        gl.clear_color(0.0, 0.25, 0.5, 1.0);
        gl.clear_depth(1.0);
        gl.clear(ClearBuffers::COLOR | ClearBuffers::DEPTH);
        gl.enable(Capability::DepthTest);
        gl.disable(Capability::Blend);
        gl.viewport(0, -1, 640, 480);
        gl.depth_func(Compare::LessOrEqual);
        gl.blend_func(BlendFactor::SourceAlpha, BlendFactor::OneMinusSourceAlpha);
        gl.cull_face(Face::Back);
        gl.front_face(Winding::Clockwise);
        gl.draw_arrays(Primitive::Triangles, 0, 3);
        gl.draw_elements(Primitive::Lines, 6, IndexType::UnsignedShort, 2);
        gl.bind_buffer(BufferTarget::Array, None);
        drop((buffer, vertices, program, texture, location));
        gl.take()
    }

    /// The same bytes are played back in `driver_module/src_js/api/command/webgl.test.ts`:
    /// change the format and one of the two tests fails, whichever side was changed.
    const FIXTURE: &[u8] = &[
        1, 63, 2, 1, 0, 0, 0, 4, 146, 136, 0, 0, 1, 0, 0, 0, 5, 146, 136, 0, 0, 228, 136, 0, 0, 8,
        0, 0, 0, 0, 0, 192, 63, 0, 0, 0, 192, 6, 147, 136, 0, 0, 2, 0, 0, 0, 4, 0, 0, 0, 7, 0, 8,
        0, 7, 2, 0, 0, 0, 9, 2, 0, 0, 0, 10, 1, 0, 0, 0, 11, 1, 0, 0, 0, 3, 6, 20, 0, 0, 1, 12, 0,
        0, 0, 4, 0, 0, 0, 12, 3, 0, 0, 0, 2, 0, 0, 0, 118, 115, 3, 0, 0, 0, 102, 197, 155, 14, 3,
        0, 0, 0, 41, 4, 0, 0, 0, 3, 0, 0, 0, 1, 0, 117, 15, 4, 0, 0, 0, 0, 0, 0, 63, 16, 4, 0, 0,
        0, 0, 0, 128, 63, 0, 0, 0, 64, 17, 4, 0, 0, 0, 0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 64, 64,
        18, 4, 0, 0, 0, 0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 64, 64, 0, 0, 128, 64, 19, 4, 0, 0, 0,
        255, 255, 255, 255, 20, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 64, 64,
        0, 0, 128, 64, 0, 0, 160, 64, 0, 0, 192, 64, 0, 0, 224, 64, 0, 0, 0, 65, 21, 4, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 64, 64, 0, 0, 128, 64, 0, 0, 160, 64, 0, 0, 192,
        64, 0, 0, 224, 64, 0, 0, 0, 65, 0, 0, 16, 65, 0, 0, 32, 65, 0, 0, 48, 65, 0, 0, 64, 65, 0,
        0, 80, 65, 0, 0, 96, 65, 0, 0, 112, 65, 22, 5, 0, 0, 0, 24, 2, 0, 0, 0, 25, 5, 0, 0, 0, 26,
        1, 0, 0, 0, 1, 0, 0, 0, 4, 0, 0, 0, 255, 0, 128, 255, 27, 1, 40, 0, 0, 0, 38, 0, 0, 27, 0,
        40, 0, 0, 1, 38, 0, 0, 27, 2, 40, 0, 0, 1, 41, 0, 0, 27, 3, 40, 0, 0, 47, 129, 0, 0, 28,
        29, 0, 0, 0, 0, 0, 0, 128, 62, 0, 0, 0, 63, 0, 0, 128, 63, 30, 0, 0, 128, 63, 31, 0, 65, 0,
        0, 32, 113, 11, 0, 0, 33, 226, 11, 0, 0, 34, 0, 0, 0, 0, 255, 255, 255, 255, 128, 2, 0, 0,
        224, 1, 0, 0, 35, 3, 2, 0, 0, 36, 2, 3, 0, 0, 3, 3, 0, 0, 37, 5, 4, 0, 0, 38, 0, 9, 0, 0,
        39, 4, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 40, 1, 0, 0, 0, 6, 0, 0, 0, 3, 20, 0, 0, 2, 0, 0,
        0, 4, 146, 136, 0, 0, 0, 0, 0, 0, 3, 1, 0, 0, 0, 8, 2, 0, 0, 0, 13, 3, 0, 0, 0, 23, 5, 0,
        0, 0, 42, 4, 0, 0, 0,
    ];

    #[test]
    fn matches_the_cross_language_fixture() {
        assert_eq!(
            fixture(),
            FIXTURE,
            "the format changed - update webgl.test.ts to match, or this is a bug"
        );
    }

    /// The `WebGl` commands of a batch, with the canvases created before them.
    fn batch(run: impl FnOnce()) -> Vec<(&'static str, Vec<u8>)> {
        log_start();
        transaction(|_| run());
        log_take()
            .into_iter()
            .filter_map(|command| match command {
                DriverDomCommand::CreateNode { name, .. } if name.as_str() == "canvas" => {
                    Some(("canvas", Vec::new()))
                }
                DriverDomCommand::WebGl { commands, .. } => Some(("webgl", commands)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn the_calls_of_a_transaction_follow_their_canvas_in_one_command() {
        let mut gl = None;
        let sent = batch(|| {
            let canvas = dom_element! { <canvas /> };
            let context = WebGl::new(&canvas.get_ref());
            context.clear(ClearBuffers::COLOR);
            context.draw_arrays(Primitive::Points, 0, 1);
            gl = Some((canvas, context));
        });

        let mut calls = vec![code::CONTEXT, WebGlOptions::default().flags()];
        calls.extend([code::CLEAR, 0, 0x40, 0, 0]);
        calls.extend([code::DRAW_ARRAYS, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0]);
        assert_eq!(sent, vec![("canvas", vec![]), ("webgl", calls)]);

        assert_eq!(batch(|| {}), vec![], "nothing more to send");
    }

    #[test]
    fn an_object_dropped_is_let_go_and_a_context_dropped_sends_nothing() {
        let canvas = dom_element! { <canvas /> };
        let gl = WebGl::new(&canvas.get_ref());
        let buffer = gl.create_buffer();
        batch(|| {});

        let sent = batch(|| drop(buffer));
        assert_eq!(sent, vec![("webgl", vec![code::DELETE_BUFFER, 1, 0, 0, 0])]);

        let texture = gl.create_texture();
        drop(gl);
        assert_eq!(batch(|| drop(texture)), vec![], "the context is gone");
    }
}
