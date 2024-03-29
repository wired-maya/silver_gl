use std::rc::Rc;
use cgmath::{Matrix4, vec3};
use crate::model_utils::create_quad;

use super::{GlError, Texture, RenderBuffer, MultiBindModel, ModelTrait, ShaderProgram, gl};

pub struct Framebuffer {
    id: u32,
    textures: Vec<Rc<Texture>>,
    draw_buffers: Vec<gl::types::GLenum>,
    quad: MultiBindModel, // Uses this since each FB has a separate 1 mesh quad
    width: i32,
    height: i32,
    pub render_buffer: Option<RenderBuffer>
}

impl Framebuffer {
    pub fn new(
        width: i32,
        height: i32,
        tex_num: usize,
        has_rb: bool
    ) -> Result<Framebuffer, GlError> {
        let mut framebuffer = Framebuffer::new_default(width, height);

        unsafe {
            gl::CreateFramebuffers(1, &mut framebuffer.id);
        }
        
        // Set up renderbuffer, all these assume framebuffer is bound
        framebuffer.gen_textures(tex_num);
        if has_rb { framebuffer.gen_render_buffer() };
        framebuffer.check_status()?;

        Ok(framebuffer)
    }

    pub fn new_default(width: i32, height: i32) -> Framebuffer {
        // Create quad model for framebuffer
        let model_transforms = vec![Matrix4::<f32>::from_translation(vec3(0.0, 0.0, 0.0))];
        let quad = create_quad(model_transforms);

        Framebuffer {
            id: 0,
            textures: Vec::new(),
            draw_buffers: Vec::new(),
            quad,
            width,
            height,
            render_buffer: None,
        }
    }

    pub fn gen_textures(&mut self, n: usize) {
        unsafe {
            for i in 0..n {
                let texture = Texture::new_mut(self.width, self.height);
                let attachment = gl::COLOR_ATTACHMENT0 + i as u32;

                // Bind to framebuffer
                gl::NamedFramebufferTexture(
                    self.id,
                    attachment,
                    texture.get_id(),
                    0
                );
                
                self.textures.push(Rc::new(texture));
                self.draw_buffers.push(attachment);
            }

            gl::NamedFramebufferDrawBuffers(
                self.id,
                self.draw_buffers.len() as i32,
                self.draw_buffers.as_ptr()
            );
        }
    }

    pub fn gen_render_buffer(&mut self) {
        let render_buffer = RenderBuffer::new(self.width, self.height);

        unsafe {
            gl::NamedFramebufferRenderbuffer(
                self.get_id(),
                gl::DEPTH_STENCIL_ATTACHMENT,
                gl::RENDERBUFFER,
                render_buffer.get_id()
            );
        }

        self.render_buffer = Some(render_buffer);
    }

    pub fn check_status(&self) -> Result<(), GlError> {
        unsafe {
            if gl::CheckNamedFramebufferStatus(self.id, gl::FRAMEBUFFER) == gl::FRAMEBUFFER_COMPLETE {
                Ok(())
            } else {
                Err(GlError::FramebufferNotComplete(self.id))
            }
        }
    }

    pub fn get_link(&self) -> Vec<Rc<Texture>> {
        let mut result = Vec::new();

        for texture in self.textures.iter() {
            result.push(Rc::clone(&texture));
        }

        result
    }

    pub fn link_to(&mut self, output: Vec<Rc<Texture>>) {
        for texture in output {
            self.link_push(texture);
        }
    }

    // framebuffer output -> self input
    // Does not clear to allow for multiple linking in a render pipeline,
    // AKA you have to do it
    pub fn link_to_fb(&mut self, framebuffer: &Framebuffer) {
        self.link_to(framebuffer.get_link());
    }

    pub fn link_push(&mut self, texture: Rc<Texture>) {
        self.quad.meshes[0].diffuse_textures.push(texture);
    }

    pub fn unlink(&mut self) {
        self.quad.meshes[0].diffuse_textures.clear();
    }

    // Get output texture at index
    pub fn get(&self, index: usize) -> Option<Rc<Texture>> {
        if let Some(texture) = self.textures.get(index) {
            Some(Rc::clone(texture))
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.textures.len()
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.id);
        }
    }

    pub fn unbind() {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }

    // Requires shader program bind
    pub fn draw(&self, shader_program: &ShaderProgram) -> Result<(), GlError> {
        self.bind();

        self.quad.draw(shader_program)?;

        Ok(())
    }

    pub fn get_size(&self) -> (i32, i32) {
        return (self.width, self.height);
    }

    pub fn set_size(&mut self, width: i32, height: i32) -> Result<(), GlError> {
        self.width = width;
        self.height = height;

        unsafe {
            for texture in self.textures.iter() {
                texture.resize(width, height)?;
            }

            if let Some(rbo) = &self.render_buffer {
                rbo.resize(width, height);
            }
        }

        Ok(())
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.id);
        }
    }
}