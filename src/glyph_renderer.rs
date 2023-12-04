use glow::{HasContext, NativeBuffer, NativeProgram, NativeVertexArray};

use crate::{gl_util, glyph_cache::GlyphCache, GlError};

unsafe fn shader_input_to_u8_slice(input: &[ShaderInput]) -> &[u8] {
    core::slice::from_raw_parts(input.as_ptr() as *const u8, std::mem::size_of_val(input))
}

unsafe fn generate_square_buffer(gl: &glow::Context) -> NativeBuffer {
    #[rustfmt::skip]
    let vertex_data: &[ShaderInput] = &[
        ShaderInput {
            vert_coord: [-1.0, -1.0],
            tex_coord: [0.0, 0.0],
        },
        ShaderInput {
            vert_coord: [-1.0, 1.0],
            tex_coord: [0.0, 1.0],
        },
        ShaderInput {
            vert_coord: [1.0, -1.0],
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

#[repr(C, packed)]
struct ShaderInput {
    vert_coord: [f32; 2],
    tex_coord: [f32; 2],
}

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

enum CursorMovement {
    Repeat(f32),
    Horiz(f32),
    Vert(f32),
}

pub struct GlyphRenderer<'a> {
    program: NativeProgram,
    vao: NativeVertexArray,
    vbo: NativeBuffer,
    gl: &'a glow::Context,
    glyph_cache: &'a mut GlyphCache,
    aspect_loc: <glow::Context as HasContext>::UniformLocation,
}

impl<'a> GlyphRenderer<'a> {
    pub fn new(
        gl: &'a glow::Context,
        glyph_cache: &'a mut GlyphCache,
    ) -> Result<GlyphRenderer<'a>, GlError> {
        unsafe {
            let program = gl_util::compile_program(
                gl,
                include_str!("glsl/vertex.glsl"),
                include_str!("glsl/sdf_fragment.glsl"),
            );

            let vao = gl.create_vertex_array().map_err(GlError)?;
            gl.bind_vertex_array(Some(vao));

            let vbo = generate_square_buffer(gl);
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

            assert!(std::mem::size_of::<ShaderInput>() == 16);

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

            Ok(GlyphRenderer {
                program,
                vao,
                vbo,
                gl,
                glyph_cache,
                aspect_loc,
            })
        }
    }

    fn scale(&self) -> f32 {
        1.0f32 / 32.0 / self.glyph_cache.pixel_size() as f32
    }

    pub fn line_height(&self) -> f32 {
        400.0 * self.scale()
    }

    fn render_char(&mut self, c: char, x: f32, y: f32, aspect: f32) -> CursorMovement {
        let scale = self.scale();
        let line_height = self.line_height();

        if c == '\n' {
            return CursorMovement::Vert(-line_height);
        }

        let gl = self.gl;

        let g_info = self.glyph_cache.get_character(gl, c).unwrap();
        let x = x + g_info.left as f32 * scale;
        let y = y + (g_info.top - g_info.height) as f32 * scale;
        let w = g_info.width as f32 * scale;
        let h = g_info.height as f32 * scale;

        if x + w > 1.0 {
            return CursorMovement::Repeat(-line_height);
        }

        unsafe {
            gl.use_program(Some(self.program));
            gl.bind_vertex_array(Some(self.vao));

            let verts: &[ShaderInput] = &[
                ShaderInput {
                    vert_coord: [x, y],
                    tex_coord: [0.0f32, 1f32],
                },
                ShaderInput {
                    vert_coord: [x + w, y],
                    tex_coord: [1.0f32, 1.0f32],
                },
                ShaderInput {
                    vert_coord: [x, y + h],
                    tex_coord: [0.0f32, 0.0f32],
                },
                ShaderInput {
                    vert_coord: [x + w, y + h],
                    tex_coord: [1.0f32, 0.0f32],
                },
            ];

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            gl.buffer_sub_data_u8_slice(glow::ARRAY_BUFFER, 0, shader_input_to_u8_slice(verts));

            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(g_info.texture));

            gl.uniform_1_f32(Some(&self.aspect_loc), aspect);

            gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
            gl.bind_vertex_array(None);
            gl.use_program(None);
        }

        CursorMovement::Horiz(g_info.advance_x as f32 / 64.0f32 * scale)
    }

    pub fn render_str(&mut self, s: &str, x: f32, y: f32, aspect: f32) -> (f32, f32) {
        let mut advance = 0.0f32;
        let mut advance_y = 0.0f32;
        let mut it = s.chars();
        let mut c = it.next();
        loop {
            if c.is_none() {
                break;
            }

            match self.render_char(c.unwrap(), x + advance, y + advance_y, aspect) {
                CursorMovement::Vert(v) => {
                    advance_y += v;
                    advance = 0.0;
                }
                CursorMovement::Horiz(v) => advance += v,
                CursorMovement::Repeat(v) => {
                    advance_y += v;
                    advance = 0.0;
                    continue;
                }
            }
            c = it.next();
        }
        (advance, advance_y)
    }
}

impl Drop for GlyphRenderer<'_> {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.program);
            self.gl.delete_buffer(self.vbo);
            self.gl.delete_vertex_array(self.vao);
        }
    }
}
