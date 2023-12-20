use glow::{HasContext, NativeBuffer, NativeProgram, NativeTexture, NativeVertexArray};

use crate::{gl_util, GlError};

//FIXME: copy paste :(
#[repr(C, packed)]
struct ShaderInput {
    vert_coord: [f32; 2],
    tex_coord: [f32; 2],
}

//FIXME: copy paste :(
macro_rules! shader_input_offset {
    ($field:ident) => {{
        let s = ShaderInput {
            vert_coord: [0.0f32; 2],
            tex_coord: [0.0f32; 2],
        };

        unsafe {
            let coord_addr = std::ptr::addr_of!(s.$field);
            (coord_addr as *const u8).offset_from(&s as *const ShaderInput as *const u8)
        }
    }};
}

// FIXME: Copy paste :(
unsafe fn shader_input_to_u8_slice(input: &[ShaderInput]) -> &[u8] {
    core::slice::from_raw_parts(input.as_ptr() as *const u8, std::mem::size_of_val(input))
}

// FIXME: Copy paste :(
unsafe fn generate_square_buffer(gl: &glow::Context) -> NativeBuffer {
    #[rustfmt::skip]
    let vertex_data: &[ShaderInput] = &[
        ShaderInput {
            vert_coord: [0.0, 0.0],
            tex_coord: [0.0, 0.0],
        },
        ShaderInput {
            vert_coord: [0.0, 1.0],
            tex_coord: [0.0, 1.0],
        },
        ShaderInput {
            vert_coord: [1.0, 0.0],
            tex_coord: [1.0, 0.0],
        },
        ShaderInput {
            vert_coord: [1.0, 1.0],
            tex_coord: [1.0, 1.0],
        },
    ];

    let vbo = gl.create_buffer().unwrap();
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

    gl.buffer_data_u8_slice(
        glow::ARRAY_BUFFER,
        shader_input_to_u8_slice(vertex_data),
        glow::STATIC_DRAW,
    );

    vbo
}

pub struct ScreenTexPostprocessor<'a> {
    program: NativeProgram,
    vao: NativeVertexArray,
    vbo: NativeBuffer,
    gl: &'a glow::Context,
    aspect_loc: <glow::Context as HasContext>::UniformLocation,
    time_loc: <glow::Context as HasContext>::UniformLocation,
}

impl<'a> ScreenTexPostprocessor<'a> {
    pub fn new(gl: &'a glow::Context) -> Result<ScreenTexPostprocessor<'a>, GlError> {
        unsafe {
            let program = gl_util::compile_program(
                gl,
                include_str!("glsl/vertex.glsl"),
                include_str!("glsl/screen_fragment.glsl"),
            );

            let vao = gl.create_vertex_array().map_err(GlError)?;
            gl.bind_vertex_array(Some(vao));

            let vbo = generate_square_buffer(gl);
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

            const STRIDE: i32 = std::mem::size_of::<ShaderInput>() as i32;
            const VERT_COORD_OFFSET: i32 = shader_input_offset!(vert_coord) as i32;
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, STRIDE, VERT_COORD_OFFSET);
            gl.enable_vertex_attrib_array(0);

            const TEX_COORD_OFFSET: i32 = shader_input_offset!(tex_coord) as i32;
            gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, STRIDE, TEX_COORD_OFFSET);
            gl.enable_vertex_attrib_array(1);

            gl.bind_vertex_array(None);

            let aspect_loc = gl
                .get_uniform_location(program, "aspect_ratio")
                .expect("Invalid vertex shader");

            let time_loc = gl
                .get_uniform_location(program, "time")
                .expect("Invalid vertex shader");

            Ok(ScreenTexPostprocessor {
                program,
                vao,
                vbo,
                gl,
                aspect_loc,
                time_loc,
            })
        }
    }

    pub fn render(&self, tex: NativeTexture, time: f32, aspect: f32) {
        let gl = self.gl;

        unsafe {
            gl.use_program(Some(self.program));
            gl.bind_vertex_array(Some(self.vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));

            gl.uniform_1_f32(Some(&self.aspect_loc), aspect);
            gl.uniform_1_f32(Some(&self.time_loc), time * 20.0);

            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(tex));

            gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
            gl.bind_vertex_array(None);
            gl.use_program(None);
        }
    }
}

impl Drop for ScreenTexPostprocessor<'_> {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.program);
            self.gl.delete_buffer(self.vbo);
            self.gl.delete_vertex_array(self.vao);
        }
    }
}
