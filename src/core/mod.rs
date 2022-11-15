use std::collections::BTreeMap;
use std::ffi::CString;
use std::ptr;

use gl::types::{GLint, GLuint};
use glam::Vec4;
use lazy_static::lazy_static;

use self::shader::{set_uniform_int, Shader};
use crate::c_str;
use crate::math::camera::Camera;

pub mod shader;

pub struct PostProcessingShader {
    uniform_cache: BTreeMap<String, GLint>,
    pub shader: Shader,
}

impl PostProcessingShader {
    pub fn new(shader: Shader) -> Self {
        shader.use_program();
        set_uniform_int(shader.get_uniform_location(c_str!("albedo")), 0);
        set_uniform_int(shader.get_uniform_location(c_str!("emissive")), 1);
        set_uniform_int(shader.get_uniform_location(c_str!("bumpmap")), 2);

        PostProcessingShader {
            uniform_cache: BTreeMap::new(),
            shader,
        }
    }

    pub fn uniform(&mut self, name: &str) -> GLint {
        if let Some(element) = self.uniform_cache.get(name) {
            *element
        } else {
            let element = self
                .shader
                .get_uniform_location(&CString::new(name).unwrap());
            self.uniform_cache.insert(name.to_owned(), element);
            element
        }
    }

    pub fn has_uniform(&self, name: &str) -> bool {
        self.uniform_cache.contains_key(name)
    }
}

pub struct Inox2DRuntime {
    in_viewport_width: i32,
    in_viewport_height: i32,

    scene_vao: GLuint,
    scene_vbo: GLuint,

    f_buffer: GLuint,
    f_albedo: GLuint,
    f_emissive: GLuint,
    f_bump: GLuint,
    f_stencil: GLuint,

    cf_buffer: GLuint,
    cf_albedo: GLuint,
    cf_emissive: GLuint,
    cf_bump: GLuint,
    cf_stencil: GLuint,

    in_clear_color: Vec4,

    basic_scene_shader: PostProcessingShader,
    basic_scene_lighting: PostProcessingShader,
    post_processing_stack: Vec<PostProcessingShader>,

    in_camera: Camera,

    is_compositing: bool,
}

impl Inox2DRuntime {
    pub(crate) fn init_renderer(&mut self, width: i32, height: i32) {
        self.in_viewport_width = width;
        self.in_viewport_height = height;

        #[cfg(feature = "in_does_render")]
        unsafe {
            // Render framebuffer
            gl::BindTexture(gl::TEXTURE_2D, self.f_albedo);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as GLint,
                width,
                height,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                ptr::null(),
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);

            gl::BindTexture(gl::TEXTURE_2D, self.f_emissive);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as GLint,
                width,
                height,
                0,
                gl::RGBA,
                gl::FLOAT,
                ptr::null(),
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);

            gl::BindTexture(gl::TEXTURE_2D, self.f_bump);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as GLint,
                width,
                height,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                ptr::null(),
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);

            gl::BindTexture(gl::TEXTURE_2D, self.f_stencil);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::DEPTH24_STENCIL8 as GLint,
                width,
                height,
                0,
                gl::DEPTH_STENCIL,
                gl::UNSIGNED_INT_24_8,
                ptr::null(),
            );

            gl::BindFramebuffer(gl::FRAMEBUFFER, self.f_buffer);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                self.f_albedo,
                0,
            );
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT1,
                gl::TEXTURE_2D,
                self.f_emissive,
                0,
            );
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT2,
                gl::TEXTURE_2D,
                self.f_bump,
                0,
            );
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::DEPTH_STENCIL_ATTACHMENT,
                gl::TEXTURE_2D,
                self.f_stencil,
                0,
            );

            // Composite framebuffer
            gl::BindTexture(gl::TEXTURE_2D, self.cf_albedo);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as GLint,
                width,
                height,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                ptr::null(),
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);

            gl::BindTexture(gl::TEXTURE_2D, self.cf_emissive);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as GLint,
                width,
                height,
                0,
                gl::RGBA,
                gl::FLOAT,
                ptr::null(),
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);

            gl::BindTexture(gl::TEXTURE_2D, self.cf_bump);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as GLint,
                width,
                height,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                ptr::null(),
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);

            gl::BindTexture(gl::TEXTURE_2D, self.cf_stencil);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::DEPTH24_STENCIL8 as GLint,
                width,
                height,
                0,
                gl::DEPTH_STENCIL,
                gl::UNSIGNED_INT_24_8,
                ptr::null(),
            );

            gl::BindFramebuffer(gl::FRAMEBUFFER, self.cf_buffer);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                self.cf_albedo,
                0,
            );
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT1,
                gl::TEXTURE_2D,
                self.cf_emissive,
                0,
            );
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT2,
                gl::TEXTURE_2D,
                self.cf_bump,
                0,
            );
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::DEPTH_STENCIL_ATTACHMENT,
                gl::TEXTURE_2D,
                self.cf_stencil,
                0,
            );

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }
}

lazy_static! {
    static ref INOX2D_RUNTIME: Inox2DRuntime = todo!();
}

pub fn in_get_viewport() -> (i32, i32) {
    (INOX2D_RUNTIME.in_viewport_width, INOX2D_RUNTIME.in_viewport_height)
}