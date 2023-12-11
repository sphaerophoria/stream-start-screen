use glow::{HasContext, NativeBuffer, NativeProgram, NativeTexture, NativeVertexArray};

use thiserror::Error;

use crate::mat::Transform;
use crate::obj_parser::{Mesh, VertAndUv};
use crate::{gl_util, GlError};

pub struct GpuMesh<'a> {
    gl: &'a glow::Context,
    vao: NativeVertexArray,
    vbo: NativeBuffer,
    ebo: NativeBuffer,
    // NOTE: Not owned, do not free
    tex: NativeTexture,
    num_elements: i32,
}

impl Drop for GpuMesh<'_> {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_vertex_array(self.vao);
            self.gl.delete_buffer(self.ebo);
            self.gl.delete_buffer(self.vbo);
        }
    }
}

#[derive(Debug, Error)]
pub enum UploadMeshError {
    #[error("failed to create vao")]
    Vao(GlError),
    #[error("failed to create vbo")]
    Vbo(GlError),
    #[error("failed to create ebo")]
    Ebo(GlError),
}

pub struct MeshRenderer<'a> {
    program: NativeProgram,
    vert_loc: u32,
    uv_loc: u32,
    model_loc: <glow::Context as HasContext>::UniformLocation,
    view_loc: <glow::Context as HasContext>::UniformLocation,
    gl: &'a glow::Context,
}

impl<'a> MeshRenderer<'a> {
    pub fn new(gl: &'a glow::Context) -> Result<MeshRenderer<'a>, GlError> {
        unsafe {
            let program = gl_util::compile_program(
                gl,
                include_str!("glsl/3d_vertex.glsl"),
                include_str!("glsl/3d_fragment.glsl"),
            );

            let vert_loc = gl
                .get_attrib_location(program, "in_vert")
                .expect("No in vert");

            let uv_loc = gl
                .get_attrib_location(program, "in_uv")
                .expect("No in vert");

            let model_loc = gl
                .get_uniform_location(program, "model")
                .expect("Invalid vertex shader");

            let view_loc = gl
                .get_uniform_location(program, "view")
                .expect("Invalid vertex shader");

            Ok(MeshRenderer {
                program,
                vert_loc,
                model_loc,
                view_loc,
                uv_loc,
                gl,
            })
        }
    }

    pub fn upload_mesh(&self, mesh: &Mesh, tex: NativeTexture) -> Result<GpuMesh, UploadMeshError> {
        unsafe {
            let gl = self.gl;

            let vao = gl
                .create_vertex_array()
                .map_err(GlError)
                .map_err(UploadMeshError::Vao)?;
            gl.bind_vertex_array(Some(vao));

            let vbo = gl
                .create_buffer()
                .map_err(GlError)
                .map_err(UploadMeshError::Vbo)?;
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                slice_arr_to_u8_slice(&mesh.vertices),
                glow::STATIC_DRAW,
            );

            let ebo = gl
                .create_buffer()
                .map_err(GlError)
                .map_err(UploadMeshError::Ebo)?;
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                slice_arr_to_u8_slice(&mesh.faces),
                glow::STATIC_DRAW,
            );

            const STRIDE: i32 = std::mem::size_of::<VertAndUv>() as i32;
            assert_eq!(STRIDE as usize, 6 * std::mem::size_of::<f32>());
            gl.vertex_attrib_pointer_f32(
                self.vert_loc,
                4,
                glow::FLOAT,
                false,
                STRIDE,
                VertAndUv::vert_offset(),
            );
            gl.enable_vertex_attrib_array(0);

            gl.vertex_attrib_pointer_f32(
                self.uv_loc,
                2,
                glow::FLOAT,
                false,
                STRIDE,
                VertAndUv::uv_offset(),
            );
            gl.enable_vertex_attrib_array(1);

            let num_elements = mesh.faces.len() * mesh.faces[0].len();

            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);

            Ok(GpuMesh {
                gl: self.gl,
                vao,
                vbo,
                ebo,
                tex,
                num_elements: num_elements.try_into().expect("Too many elements"),
            })
        }
    }

    pub fn set_camera_transform(&self, transform: &Transform) {
        unsafe {
            self.gl.use_program(Some(self.program));
            self.gl.uniform_matrix_4_f32_slice(
                Some(&self.view_loc),
                true,
                std::slice::from_raw_parts(transform.arr[0].as_ptr(), 16),
            );
            self.gl.use_program(None);
        }
    }

    pub fn render(&self, mesh: &GpuMesh, transform: &Transform) {
        let gl = self.gl;

        unsafe {
            gl.use_program(Some(self.program));
            gl.bind_vertex_array(Some(mesh.vao));

            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(mesh.tex));

            gl.uniform_matrix_4_f32_slice(
                Some(&self.model_loc),
                true,
                std::slice::from_raw_parts(transform.arr[0].as_ptr(), 16),
            );
            gl.draw_elements(glow::TRIANGLES, mesh.num_elements, glow::UNSIGNED_INT, 0);

            gl.bind_texture(glow::TEXTURE_2D, None);
            gl.bind_vertex_array(None);
            gl.use_program(None);
        }
    }
}

unsafe fn slice_arr_to_u8_slice<T>(input: &[T]) -> &[u8] {
    core::slice::from_raw_parts(input.as_ptr() as *const u8, std::mem::size_of_val(input))
}
