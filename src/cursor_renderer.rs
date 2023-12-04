use glow::{HasContext, NativeBuffer, NativeProgram, NativeVertexArray};

use crate::{gl_util, GlError};

unsafe fn f32_to_u8_slice(input: &[f32]) -> &[u8] {
    core::slice::from_raw_parts(input.as_ptr() as *const u8, std::mem::size_of_val(input))
}

unsafe fn generate_square_buffer(gl: &glow::Context) -> NativeBuffer {
    #[rustfmt::skip]
    let vertex_data: &[f32] = &[
        -1.0, -1.0,
        -1.0, 1.0,
        1.0, -1.0,
        1.0, 1.0,
    ];

    let vbo = gl.create_buffer().unwrap();
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

    gl.buffer_data_u8_slice(
        glow::ARRAY_BUFFER,
        f32_to_u8_slice(vertex_data),
        glow::STATIC_DRAW,
    );

    vbo
}

pub struct CursorRenderer<'a> {
    program: NativeProgram,
    vao: NativeVertexArray,
    vbo: NativeBuffer,
    gl: &'a glow::Context,
    aspect_loc: <glow::Context as HasContext>::UniformLocation,
}

impl<'a> CursorRenderer<'a> {
    pub fn new(gl: &'a glow::Context) -> Result<CursorRenderer<'a>, GlError> {
        unsafe {
            let program = gl_util::compile_program(
                gl,
                include_str!("glsl/color_vertex.glsl"),
                include_str!("glsl/color_fragment.glsl"),
            );

            let vao = gl.create_vertex_array().map_err(GlError)?;
            gl.bind_vertex_array(Some(vao));

            let vbo = generate_square_buffer(gl);
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

            const STRIDE: i32 = (2 * std::mem::size_of::<f32>()) as i32;
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, STRIDE, 0);
            gl.enable_vertex_attrib_array(0);

            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);

            let aspect_loc = gl
                .get_uniform_location(program, "aspect_ratio")
                .expect("Invalid vertex shader");

            Ok(CursorRenderer {
                program,
                vao,
                vbo,
                gl,
                aspect_loc,
            })
        }
    }

    pub fn render(&self, x: f32, y: f32, w: f32, h: f32, aspect: f32) {
        let gl = self.gl;

        unsafe {
            gl.use_program(Some(self.program));
            gl.bind_vertex_array(Some(self.vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));

            let verts: &[f32] = &[x, y, x + w, y, x, y + h, x + w, y + h];

            gl.buffer_sub_data_u8_slice(glow::ARRAY_BUFFER, 0, f32_to_u8_slice(verts));

            gl.uniform_1_f32(Some(&self.aspect_loc), aspect);

            gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
            gl.bind_vertex_array(None);
            gl.use_program(None);
        }
    }
}

impl Drop for CursorRenderer<'_> {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.program);
            self.gl.delete_buffer(self.vbo);
            self.gl.delete_vertex_array(self.vao);
        }
    }
}
