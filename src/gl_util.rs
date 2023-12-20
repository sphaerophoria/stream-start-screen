use glow::{HasContext, NativeFramebuffer, NativeProgram, NativeShader, NativeTexture};

use crate::GlError;

pub unsafe fn setup_depth_texture_render(
    gl: &glow::Context,
    width: i32,
    height: i32,
) -> Result<(NativeTexture, NativeFramebuffer), GlError> {
    let tex = create_tex_default_params(gl)?;
    gl.bind_texture(glow::TEXTURE_2D, Some(tex));
    gl.tex_image_2d(
        glow::TEXTURE_2D,
        0,
        glow::DEPTH_COMPONENT as i32,
        width,
        height,
        0,
        glow::DEPTH_COMPONENT,
        glow::FLOAT,
        None,
    );

    let fb = gl.create_framebuffer().map_err(GlError)?;
    gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fb));
    gl.framebuffer_texture(glow::FRAMEBUFFER, glow::DEPTH_ATTACHMENT, Some(tex), 0);

    let buffers: [u32; 1] = [glow::DEPTH_ATTACHMENT];
    gl.draw_buffers(&buffers);
    let fb_status = gl.check_framebuffer_status(glow::FRAMEBUFFER);
    if fb_status != glow::FRAMEBUFFER_COMPLETE {
        panic!("incomplete framebuffer: {fb_status:#x}");
    }
    gl.viewport(0, 0, width, height);

    Ok((tex, fb))
}

// FIXME: copy pasta with depth_texture
pub unsafe fn setup_color_texture_render(
    gl: &glow::Context,
    width: i32,
    height: i32,
) -> Result<(NativeTexture, NativeFramebuffer), GlError> {
    let tex = create_tex_default_params(gl)?;
    gl.bind_texture(glow::TEXTURE_2D, Some(tex));
    gl.tex_image_2d(
        glow::TEXTURE_2D,
        0,
        glow::RGB as i32,
        width,
        height,
        0,
        glow::RGB,
        glow::FLOAT,
        None,
    );

    let fb = gl.create_framebuffer().map_err(GlError)?;
    gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fb));
    gl.framebuffer_texture(glow::FRAMEBUFFER, glow::COLOR_ATTACHMENT0, Some(tex), 0);

    let buffers: [u32; 1] = [glow::COLOR_ATTACHMENT0];
    gl.draw_buffers(&buffers);
    let fb_status = gl.check_framebuffer_status(glow::FRAMEBUFFER);
    if fb_status != glow::FRAMEBUFFER_COMPLETE {
        panic!("incomplete framebuffer: {fb_status:#x}");
    }
    gl.viewport(0, 0, width, height);

    Ok((tex, fb))
}

pub unsafe fn create_tex_default_params(gl: &glow::Context) -> Result<NativeTexture, GlError> {
    let texture = gl.create_texture().map_err(GlError)?;

    gl.bind_texture(glow::TEXTURE_2D, Some(texture));

    gl.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_MIN_FILTER,
        glow::LINEAR as i32,
    );
    gl.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_MAG_FILTER,
        glow::LINEAR as i32,
    );

    gl.bind_texture(glow::TEXTURE_2D, None);

    Ok(texture)
}

pub unsafe fn compile_shader(
    gl: &glow::Context,
    shader_type: u32,
    shader_source: &str,
) -> NativeShader {
    let shader = gl.create_shader(shader_type).expect("Cannot create shader");
    gl.shader_source(shader, shader_source);
    gl.compile_shader(shader);
    if !gl.get_shader_compile_status(shader) {
        panic!("{}", gl.get_shader_info_log(shader));
    }
    shader
}

pub unsafe fn compile_program(
    gl: &glow::Context,
    vert_source: &str,
    frag_source: &str,
) -> NativeProgram {
    let program = gl.create_program().expect("Cannot create program");

    let vertex_shader = compile_shader(gl, glow::VERTEX_SHADER, vert_source);
    gl.attach_shader(program, vertex_shader);

    let fragment_shader = compile_shader(gl, glow::FRAGMENT_SHADER, frag_source);
    gl.attach_shader(program, fragment_shader);

    gl.link_program(program);

    if !gl.get_program_link_status(program) {
        panic!("{}", gl.get_program_info_log(program));
    }

    for shader in [vertex_shader, fragment_shader] {
        gl.detach_shader(program, shader);
        gl.delete_shader(shader);
    }

    program
}
