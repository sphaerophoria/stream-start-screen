use glow::{HasContext, NativeBuffer, NativeProgram, NativeTexture, NativeVertexArray};

use thiserror::Error;

use crate::mat::{Transform, Vec3};
use crate::obj_parser::{Mesh, VertData};
use crate::{gl_util, GlError};

pub struct GpuMesh<'a> {
    gl: &'a glow::Context,
    vao: NativeVertexArray,
    vbo: NativeBuffer,
    ebo: NativeBuffer,
    // NOTE: Not owned, do not free
    pub tex: NativeTexture,
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
    vert_loc: Option<u32>,
    uv_loc: Option<u32>,
    norm_loc: Option<u32>,
    model_loc: Option<<glow::Context as HasContext>::UniformLocation>,
    view_loc: Option<<glow::Context as HasContext>::UniformLocation>,
    view_to_light_loc: Option<<glow::Context as HasContext>::UniformLocation>,
    light_dir_loc: Option<<glow::Context as HasContext>::UniformLocation>,
    light_color_loc: Option<<glow::Context as HasContext>::UniformLocation>,
    light_tex_loc: Option<<glow::Context as HasContext>::UniformLocation>,
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

            let vert_loc = gl.get_attrib_location(program, "in_vert");

            let uv_loc = gl.get_attrib_location(program, "in_uv");

            let norm_loc = gl.get_attrib_location(program, "in_normal");

            let model_loc = gl.get_uniform_location(program, "model");

            let view_loc = gl.get_uniform_location(program, "view");

            let light_dir_loc = gl.get_uniform_location(program, "light_dir");

            let light_color_loc = gl.get_uniform_location(program, "light_color");

            let view_to_light_loc = gl.get_uniform_location(program, "view_pos_to_light_pos");

            let light_tex_loc = gl.get_uniform_location(program, "light_tex");

            Ok(MeshRenderer {
                program,
                vert_loc,
                model_loc,
                view_loc,
                light_dir_loc,
                light_color_loc,
                view_to_light_loc,
                light_tex_loc,
                uv_loc,
                norm_loc,
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

            const STRIDE: i32 = std::mem::size_of::<VertData>() as i32;
            assert_eq!(STRIDE as usize, 9 * std::mem::size_of::<f32>());

            if let Some(vert_loc) = &self.vert_loc {
                gl.vertex_attrib_pointer_f32(
                    *vert_loc,
                    4,
                    glow::FLOAT,
                    false,
                    STRIDE,
                    VertData::vert_offset(),
                );
                gl.enable_vertex_attrib_array(0);
            }

            if let Some(uv_loc) = &self.uv_loc {
                gl.vertex_attrib_pointer_f32(
                    *uv_loc,
                    2,
                    glow::FLOAT,
                    false,
                    STRIDE,
                    VertData::uv_offset(),
                );
                gl.enable_vertex_attrib_array(1);
            }

            if let Some(norm_loc) = &self.norm_loc {
                gl.vertex_attrib_pointer_f32(
                    *norm_loc,
                    3,
                    glow::FLOAT,
                    false,
                    STRIDE,
                    VertData::normal_offset(),
                );
                gl.enable_vertex_attrib_array(2);
            }

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
                self.view_loc.as_ref(),
                true,
                std::slice::from_raw_parts(transform.arr[0].as_ptr(), 16),
            );
            self.gl.use_program(None);
        }
    }

    pub fn set_view_to_light_transform(&self, transform: &Transform) {
        unsafe {
            self.gl.use_program(Some(self.program));
            self.gl.uniform_matrix_4_f32_slice(
                self.view_to_light_loc.as_ref(),
                true,
                std::slice::from_raw_parts(transform.arr[0].as_ptr(), 16),
            );
            self.gl.use_program(None);
        }
    }

    pub fn set_light_texture(&self, tex: NativeTexture) {
        unsafe {
            self.gl.use_program(Some(self.program));
            self.gl.uniform_1_i32(self.light_tex_loc.as_ref(), 1);
            self.gl.active_texture(glow::TEXTURE1);
            self.gl.bind_texture(glow::TEXTURE_2D, Some(tex));
            self.gl.use_program(None);
        }
    }

    pub fn set_light_dir(&self, dir: &Vec3) {
        unsafe {
            self.gl.use_program(Some(self.program));

            let normalized = dir.normalized();

            self.gl.uniform_3_f32(
                self.light_dir_loc.as_ref(),
                normalized.x(),
                normalized.y(),
                normalized.z(),
            );
            self.gl.use_program(None);
        }
    }

    pub fn set_light_color(&self, color: &[f32; 3]) {
        unsafe {
            self.gl.use_program(Some(self.program));

            self.gl
                .uniform_3_f32(self.light_color_loc.as_ref(), color[0], color[1], color[2]);
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
                self.model_loc.as_ref(),
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
