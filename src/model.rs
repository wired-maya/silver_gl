use cgmath::Matrix4;
use memoffset::offset_of;
use crate::{Buffer, DrawCommand};
use super::{ShaderProgram, Mesh, Vertex, GlError, VertexArray, gl, model_utils::calc_vertex_tangents};

pub trait ModelTrait {
    fn draw(&self, shader_program: &ShaderProgram) -> Result<(), GlError>;
    fn get_transform_array_mut(&mut self) -> &mut Buffer<Matrix4<f32>>;
    fn get_transform_array(&self) -> &Buffer<Matrix4<f32>>;
    fn get_meshes_mut(&mut self) -> &mut Vec<Mesh>;
    fn get_meshes(&self) -> &Vec<Mesh>;
}

pub trait ModelCreateTrait {
    fn new(vertices: Vec<Vertex>, indices: Vec<u32>, model_transform: Vec<Matrix4<f32>>, meshes: Vec<Mesh>) -> Self;
}

pub struct MultiBindModel {
    pub meshes: Vec<Mesh>,
    pub vertex_array: VertexArray,
    pub vertex_buffer: Buffer<Vertex>,
    pub element_buffer: Buffer<u32>,
    pub transform_buffer: Buffer<Matrix4<f32>>,
}

impl ModelCreateTrait for MultiBindModel {
    fn new(
        mut vertices: Vec<Vertex>,
        mut indices: Vec<u32>,
        model_transforms: Vec<Matrix4<f32>>,
        meshes: Vec<Mesh>
    ) -> Self {
        let mut model = Self {
            meshes,
            vertex_array: VertexArray::new(),
            vertex_buffer: Buffer::new(),
            element_buffer: Buffer::new(),
            transform_buffer: Buffer::new()
        };

        calc_vertex_tangents(&mut vertices, &mut indices);
        model.setup_model(vertices, indices);
        model.setup_transform_attribute(model_transforms);

        model
    }
}

impl MultiBindModel {
    pub fn setup_model(&mut self, vertices: Vec<Vertex>, indices: Vec<u32>) {
        self.vertex_array.add_vertex_buffer(&mut self.vertex_buffer);
        self.vertex_array.set_element_buffer(&mut self.element_buffer);

        self.vertex_array.add_attrib(&mut self.vertex_buffer, 3, offset_of!(Vertex, position) as u32, gl::FLOAT);
        self.vertex_array.add_attrib(&mut self.vertex_buffer, 3, offset_of!(Vertex, normal) as u32, gl::FLOAT);
        self.vertex_array.add_attrib(&mut self.vertex_buffer, 2, offset_of!(Vertex, tex_coord) as u32, gl::FLOAT);
        self.vertex_array.add_attrib(&mut self.vertex_buffer, 3, offset_of!(Vertex, tangent) as u32, gl::FLOAT);
        self.vertex_array.add_attrib(&mut self.vertex_buffer, 3, offset_of!(Vertex, bitangent) as u32, gl::FLOAT);

        self.vertex_buffer.set_data(vertices);
        self.element_buffer.set_data(indices);
    }
    
    pub fn setup_transform_attribute(&mut self, model_transforms: Vec<Matrix4<f32>>) {
        self.vertex_array.add_vertex_buffer(&mut self.transform_buffer);
        self.vertex_array.add_attrib_divisor(&mut self.transform_buffer, 4);
        self.transform_buffer.set_data_mut(model_transforms);
    }
}

// TODO: can simply draw same vertices by providing same offset in each mesh
// TODO: find a way to make this work with different transforms
impl ModelTrait for MultiBindModel {
    fn draw(&self, shader_program: &ShaderProgram) -> Result<(), GlError> {
        unsafe {
            self.vertex_array.bind();

            for mesh in &self.meshes {
                mesh.set_textures(shader_program)?;
                self.vertex_array.draw_elements_offset(
                    mesh.get_count(),
                    mesh.get_offset(),
                    self.transform_buffer.len() as i32
                );
    
                // Set back to defaults once configured
                gl::ActiveTexture(gl::TEXTURE0);
            }

            gl::BindVertexArray(0);
        }

        Ok(())
    }

    fn get_transform_array_mut(&mut self) -> &mut Buffer<Matrix4<f32>> { &mut self.transform_buffer }
    fn get_transform_array(&self) -> &Buffer<Matrix4<f32>> { &self.transform_buffer }
    fn get_meshes_mut(&mut self) -> &mut Vec<Mesh> { &mut self.meshes }
    fn get_meshes(&self) -> &Vec<Mesh> { &self.meshes }
}

pub struct BindlessModel {
    pub meshes: Vec<Mesh>,
    // TODO: rename these to something more descriptive
    pub vertex_array: VertexArray,
    pub vertex_buffer: Buffer<Vertex>,
    pub element_buffer: Buffer<u32>,
    pub transform_buffer: Buffer<Matrix4<f32>>,
    pub command_buffer: Buffer<DrawCommand>
}

impl ModelCreateTrait for BindlessModel {
    fn new(
        mut vertices: Vec<Vertex>,
        mut indices: Vec<u32>,
        model_transforms: Vec<Matrix4<f32>>,
        meshes: Vec<Mesh>
    ) -> Self {
        let mut model = Self {
            meshes,
            vertex_array: VertexArray::new(),
            vertex_buffer: Buffer::new(),
            element_buffer: Buffer::new(),
            transform_buffer: Buffer::new(),
            command_buffer: Buffer::new()
        };

        // TODO: generate draw calls and add them
        // TODO: to buffer

        calc_vertex_tangents(&mut vertices, &mut indices);
        model.setup_model(vertices, indices);
        model.setup_transform_attribute(model_transforms);

        model
    }
}

impl BindlessModel {
    pub fn setup_model(&mut self, vertices: Vec<Vertex>, indices: Vec<u32>) {
        self.vertex_array.add_vertex_buffer(&mut self.vertex_buffer);
        self.vertex_array.set_element_buffer(&mut self.element_buffer);

        self.vertex_array.add_attrib(&mut self.vertex_buffer, 3, offset_of!(Vertex, position) as u32, gl::FLOAT);
        self.vertex_array.add_attrib(&mut self.vertex_buffer, 3, offset_of!(Vertex, normal) as u32, gl::FLOAT);
        self.vertex_array.add_attrib(&mut self.vertex_buffer, 2, offset_of!(Vertex, tex_coord) as u32, gl::FLOAT);
        self.vertex_array.add_attrib(&mut self.vertex_buffer, 3, offset_of!(Vertex, tangent) as u32, gl::FLOAT);
        self.vertex_array.add_attrib(&mut self.vertex_buffer, 3, offset_of!(Vertex, bitangent) as u32, gl::FLOAT);

        self.vertex_buffer.set_data(vertices);
        self.element_buffer.set_data(indices);
    }
    
    pub fn setup_transform_attribute(&mut self, model_transforms: Vec<Matrix4<f32>>) {
        self.vertex_array.add_vertex_buffer(&mut self.transform_buffer);
        self.vertex_array.add_attrib_divisor(&mut self.transform_buffer, 4);
        self.transform_buffer.set_data_mut(model_transforms);
    }
}

impl ModelTrait for BindlessModel {
    // TODO: work on making this work with textures so there is one draw call
    // TODO: Use bindless textures and ubos to do this in one big draw call
    // TODO: Check if those extensions are supported, if not, just draw
    // TODO: each mesh individually like normal.
    // TODO: https://litasa.github.io/blog/2017/09/04/OpenGL-MultiDrawIndirect-with-Individual-Textures
    // Panics if there is no cbo present in the model
    fn draw(&self, shader_program: &ShaderProgram) -> Result<(), GlError> {
        unsafe {
            self.vertex_array.bind();
            // TODO: Generic buffer bind function?
            gl::BindBuffer(gl::DRAW_INDIRECT_BUFFER, self.command_buffer.get_id());

            for mesh in &self.meshes {
                mesh.set_textures(shader_program)?;
                self.vertex_array.draw_elements_offset(
                    mesh.get_count(),
                    mesh.get_offset(),
                    self.transform_buffer.len() as i32
                );
    
                // Set back to defaults once configured
                gl::ActiveTexture(gl::TEXTURE0);
            }

            gl::BindVertexArray(0);
        }

        Ok(())
    }

    fn get_transform_array_mut(&mut self) -> &mut Buffer<Matrix4<f32>> { &mut self.transform_buffer }
    fn get_transform_array(&self) -> &Buffer<Matrix4<f32>> { &&self.transform_buffer }
    fn get_meshes_mut(&mut self) -> &mut Vec<Mesh> { &mut self.meshes }
    fn get_meshes(&self) -> &Vec<Mesh> { &self.meshes }
}