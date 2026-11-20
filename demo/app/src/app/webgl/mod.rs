//! A cube turning, drawn with WebGL 2.
//!
//! Its vertices and their colors go to the browser once, into buffers. Then, before every frame
//! the browser draws, the cube's turn is worked out in Rust and drawn with a few calls: a
//! matrix for a uniform, and one draw of the cube's 36 corners of triangles.
//!
//! In its corner, how many frames the browser drew in the last second: a `Value` set from the
//! same callback, once a second, so the page changes only then.

use std::{cell::Cell, f32::consts::PI};

use vertigo::{
    Css, DomElementRef, Value, component, css, dom, dom_element, get_driver,
    webgl::{
        Buffer, BufferTarget, BufferUsage, Capability, ClearBuffers, DataType, IndexType,
        Primitive, Program, VertexArray, WebGl, WebGlOptions,
    },
};

#[component]
pub fn WebGlDemo() {
    let canvas = dom_element! { <canvas css={css_canvas()} /> };
    let cube = Cube::new(&canvas.get_ref());

    let fps = Value::new(None);
    let counter = FrameCounter::default();
    canvas.append_drop_resource(get_driver().on_animation_frame({
        let fps = fps.clone();
        move |time| {
            cube.draw(time);
            if let Some(count) = counter.frame(time) {
                fps.set(Some(count));
            }
        }
    }));
    let fps = fps.map(|fps| match fps {
        Some(fps) => format!("{fps} FPS"),
        None => "… FPS".to_string(),
    });

    dom! {
        <div css={css_wrapper()}>
            <p>"A cube drawn with WebGL 2, its turn worked out in Rust for every frame."</p>
            <div css={css_frame()}>
                {canvas}
                <span css={css_fps()}>{fps}</span>
            </div>
        </div>
    }
}

/// Frames counted for a second at a time.
#[derive(Default)]
struct FrameCounter {
    /// When the second being counted began, in milliseconds.
    since: Cell<Option<f64>>,
    frames: Cell<u32>,
}

impl FrameCounter {
    /// A frame at `time`. Once a second has gone since the count began, how many frames came in
    /// it, and the count begins again.
    fn frame(&self, time: f64) -> Option<u32> {
        let since = self.since.get().unwrap_or(time);
        self.since.set(Some(since));
        self.frames.set(self.frames.get() + 1);

        let elapsed = time - since;
        if elapsed < 1000.0 {
            return None;
        }

        let count = (f64::from(self.frames.get() - 1) * 1000.0 / elapsed).round() as u32;
        self.since.set(Some(time));
        self.frames.set(1);
        Some(count)
    }
}

const VERTEX: &str = "#version 300 es
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 color;
uniform mat4 transform;
out vec3 shade;

void main() {
    gl_Position = transform * vec4(position, 1.0);
    shade = color;
}";

const FRAGMENT: &str = "#version 300 es
precision mediump float;
in vec3 shade;
out vec4 pixel;

void main() {
    pixel = vec4(shade, 1.0);
}";

type Vector = [f32; 3];

/// The faces of the cube: a corner, the two edges from it, and a color.
const FACES: [(Vector, Vector, Vector, Vector); 6] = [
    (
        [-1.0, -1.0, 1.0],
        [2.0, 0.0, 0.0],
        [0.0, 2.0, 0.0],
        [0.90, 0.22, 0.21],
    ),
    (
        [1.0, -1.0, -1.0],
        [-2.0, 0.0, 0.0],
        [0.0, 2.0, 0.0],
        [0.99, 0.85, 0.21],
    ),
    (
        [-1.0, -1.0, -1.0],
        [0.0, 0.0, 2.0],
        [0.0, 2.0, 0.0],
        [0.26, 0.63, 0.28],
    ),
    (
        [1.0, -1.0, 1.0],
        [0.0, 0.0, -2.0],
        [0.0, 2.0, 0.0],
        [0.12, 0.53, 0.90],
    ),
    (
        [-1.0, 1.0, 1.0],
        [2.0, 0.0, 0.0],
        [0.0, 0.0, -2.0],
        [0.56, 0.14, 0.67],
    ),
    (
        [-1.0, -1.0, -1.0],
        [2.0, 0.0, 0.0],
        [0.0, 0.0, 2.0],
        [0.98, 0.55, 0.0],
    ),
];

struct Cube {
    gl: WebGl,
    program: Program,
    vertices: VertexArray,
    // Kept for as long as the cube is drawn, as the vertex array reads them.
    _buffers: [Buffer; 2],
}

impl Cube {
    fn new(canvas: &DomElementRef) -> Self {
        // So that a test can read the pixels back between frames.
        let gl = WebGl::with_options(
            canvas,
            WebGlOptions {
                preserve_drawing_buffer: true,
                ..WebGlOptions::default()
            },
        );
        let program = gl.create_program(VERTEX, FRAGMENT);

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

        let vertices = gl.create_vertex_array();
        gl.bind_vertex_array(Some(&vertices));

        let corner_buffer = gl.create_buffer();
        gl.bind_buffer(BufferTarget::Array, Some(&corner_buffer));
        gl.buffer_data(BufferTarget::Array, &corners, BufferUsage::StaticDraw);
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer(0, 3, DataType::Float, false, 24, 0);
        gl.enable_vertex_attrib_array(1);
        gl.vertex_attrib_pointer(1, 3, DataType::Float, false, 24, 12);

        let index_buffer = gl.create_buffer();
        gl.bind_buffer(BufferTarget::ElementArray, Some(&index_buffer));
        gl.buffer_data(
            BufferTarget::ElementArray,
            &indices,
            BufferUsage::StaticDraw,
        );

        gl.bind_vertex_array(None);
        gl.enable(Capability::DepthTest);
        gl.enable(Capability::CullFace);
        gl.clear_color(0.95, 0.95, 0.97, 1.0);

        Self {
            gl,
            program,
            vertices,
            _buffers: [corner_buffer, index_buffer],
        }
    }

    /// The cube at `time`, in milliseconds: a turn around one axis every 6 seconds, and around
    /// another every 10.
    fn draw(&self, time: f64) {
        let seconds = (time / 1000.0) as f32;
        let transform = multiply(
            &multiply(
                &perspective(PI / 4.0, 1.0, 0.1, 100.0),
                &translation(0.0, 0.0, -6.0),
            ),
            &multiply(
                &rotation_x(seconds * PI / 5.0),
                &rotation_y(seconds * PI / 3.0),
            ),
        );

        let gl = &self.gl;
        gl.clear(ClearBuffers::COLOR | ClearBuffers::DEPTH);
        gl.use_program(Some(&self.program));
        gl.uniform_matrix_4f("transform", &transform);
        gl.bind_vertex_array(Some(&self.vertices));
        gl.draw_elements(Primitive::Triangles, 36, IndexType::UnsignedShort, 0);
    }
}

// Matrices of 4 x 4, column after column, as WebGL takes them.

type Matrix = [f32; 16];

fn multiply(a: &Matrix, b: &Matrix) -> Matrix {
    let mut product = [0.0; 16];
    for column in 0..4 {
        for row in 0..4 {
            product[column * 4 + row] = (0..4).map(|k| a[k * 4 + row] * b[column * 4 + k]).sum();
        }
    }
    product
}

/// Seen through a lens of `fov` radians from top to bottom, from `near` to `far`.
#[rustfmt::skip]
fn perspective(fov: f32, aspect: f32, near: f32, far: f32) -> Matrix {
    let f = 1.0 / (fov / 2.0).tan();
    let depth = near - far;
    [
        f / aspect, 0.0, 0.0, 0.0,
        0.0, f, 0.0, 0.0,
        0.0, 0.0, (far + near) / depth, -1.0,
        0.0, 0.0, 2.0 * far * near / depth, 0.0,
    ]
}

#[rustfmt::skip]
fn translation(x: f32, y: f32, z: f32) -> Matrix {
    [
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        x, y, z, 1.0,
    ]
}

#[rustfmt::skip]
fn rotation_x(angle: f32) -> Matrix {
    let (sin, cos) = angle.sin_cos();
    [
        1.0, 0.0, 0.0, 0.0,
        0.0, cos, sin, 0.0,
        0.0, -sin, cos, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ]
}

#[rustfmt::skip]
fn rotation_y(angle: f32) -> Matrix {
    let (sin, cos) = angle.sin_cos();
    [
        cos, 0.0, -sin, 0.0,
        0.0, 1.0, 0.0, 0.0,
        sin, 0.0, cos, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ]
}

fn css_wrapper() -> Css {
    css! {"
        margin: 10px 0;
    "}
}

/// Around the canvas, as big as it is, for the count to sit in its corner.
fn css_frame() -> Css {
    css! {"
        position: relative;
        display: inline-block;
    "}
}

fn css_fps() -> Css {
    css! {"
        position: absolute;
        top: 6px;
        right: 6px;
        padding: 2px 6px;
        border-radius: 3px;
        background: rgba(0, 0, 0, 0.6);
        color: white;
        font: 12px monospace;
        pointer-events: none;
    "}
}

fn css_canvas() -> Css {
    css! {"
        display: block;
        width: 320px;
        height: 320px;
        border: 1px solid #ccc;
    "}
}

#[cfg(test)]
mod tests {
    use super::FrameCounter;

    #[test]
    fn counts_the_frames_of_each_second() {
        let counter = FrameCounter::default();
        let at_60 = |frame: u32| 500.0 + f64::from(frame) * 1000.0 / 60.0;

        let counts: Vec<_> = (0..=120)
            .filter_map(|frame| counter.frame(at_60(frame)))
            .collect();
        assert_eq!(counts, vec![60, 60], "a count each second, none before");

        // Slower: a frame every 50 ms.
        let counts: Vec<_> = (1..=20)
            .filter_map(|frame| counter.frame(at_60(120) + f64::from(frame) * 50.0))
            .collect();
        assert_eq!(counts, vec![20]);
    }
}
