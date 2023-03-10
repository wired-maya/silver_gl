use super::ShaderProgram;
use super::{GlError, gl};

// UBO can have multiple types of data, so it doesn't have a type
pub struct UniformBuffer {
    id: u32,
    name: String,
    buffer_size: isize
}

impl UniformBuffer {
    pub fn new(shader_programs: Vec<&ShaderProgram>, name: &str, buffer_size: isize) -> Result<UniformBuffer, GlError> {
        let mut uniform_buffer = UniformBuffer {
            id: 0,
            name: String::from(name),
            buffer_size
        };

        for shader_program in shader_programs.iter() {
            uniform_buffer.register_shader_program(shader_program)?;
        }

        uniform_buffer.create_ubo();

        Ok(uniform_buffer)
    }

    pub fn register_shader_program(&self, shader_program: &ShaderProgram) -> Result<(), GlError> {
        shader_program.bind_to_ubo(self.name.as_str())
    }

    pub fn create_ubo(&mut self) {
        unsafe {
            gl::CreateBuffers(1, &mut self.id);
            gl::NamedBufferData(self.id, self.buffer_size, std::ptr::null(), gl::DYNAMIC_DRAW);
            gl::BindBufferRange(gl::UNIFORM_BUFFER, 0, self.id, 0, self.buffer_size);
        }
    }

    pub fn bind_ubo(&self) {
        unsafe {
            gl::BindBufferRange(gl::UNIFORM_BUFFER, 0, self.id, 0, self.buffer_size);
        }
    }

    pub fn write_data<T>(&self, data: *const gl::types::GLvoid, offset: u32) {
        unsafe {
            gl::NamedBufferSubData(self.id, offset as isize, std::mem::size_of::<T>() as isize, data);
        }
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }
}

impl Drop for UniformBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}