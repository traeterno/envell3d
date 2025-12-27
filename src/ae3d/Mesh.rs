use crate::ae3d::{glTF::GLTF, Camera::Drawable, Transformable::Transformable3D};

#[derive(Default, Debug)]
pub struct Mesh
{
	ts: Transformable3D,
	vao: u32,
	vbo: u32,
	ebo: u32,
	elements: i32,
	texture: u32
}

impl Mesh
{
	pub fn new() -> Self
	{
		let mut vao = 0;
		let mut vbo = 0;
		let mut ebo = 0;
		unsafe
		{
			gl::GenVertexArrays(1, &mut vao);
			gl::GenBuffers(1, &mut vbo);
			gl::GenBuffers(1, &mut ebo);

			gl::BindVertexArray(vao);
			gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
			gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);

			gl::EnableVertexAttribArray(0);
			gl::EnableVertexAttribArray(1);
			gl::EnableVertexAttribArray(2);

			gl::VertexAttribPointer(
				0, 3, gl::FLOAT, gl::FALSE,
				(9 * size_of::<f32>()) as i32, 0 as _
			);

			gl::VertexAttribPointer(
				1, 4, gl::FLOAT, gl::FALSE,
				(9 * size_of::<f32>()) as i32,
				(3 * size_of::<f32>()) as _
			);

			gl::VertexAttribPointer(
				2, 2, gl::FLOAT, gl::FALSE,
				(9 * size_of::<f32>()) as i32,
				(7 * size_of::<f32>()) as _
			);
		}
		Self
		{
			ts: Transformable3D::new(),
			vao, vbo, ebo,
			elements: 0, texture: 0
		}
	}

	pub fn fromGLTF(gltf: &GLTF, id: usize) -> Mesh
	{
		let (
			vertices, normals, elements,
			joints, uvs, texture
		) = gltf.mesh(id);
		
		let mut buffer: Vec<f32> = vec![];

		for i in 0..(vertices.len() / 3)
		{
			buffer.push(vertices[i * 3]);
			buffer.push(vertices[i * 3 + 1]);
			buffer.push(vertices[i * 3 + 2]);
			if normals.len() == 0 { buffer.append(&mut vec![0.0; 3]); }
			else
			{
				buffer.push(normals[i * 3]);
				buffer.push(normals[i * 3 + 1]);
				buffer.push(normals[i * 3 + 2]);
			}
			if joints.len() == 0 { buffer.push(-1.0); }
			else { buffer.push(joints[i]); }
			if uvs.len() == 0 { buffer.append(&mut vec![0.0; 2]); }
			else
			{
				buffer.push(uvs[i * 2]);
				buffer.push(uvs[i * 2 + 1]);
			}
		}

		let mut m = Mesh::new();
		m.elements = elements.len() as i32;
		m.texture = texture;

		unsafe
		{
			gl::BindVertexArray(m.vao);
			gl::BindBuffer(gl::ARRAY_BUFFER, m.vbo);
			gl::BufferData(gl::ARRAY_BUFFER,
				(buffer.len() * size_of::<f32>()) as isize,
				buffer.as_ptr() as *const _,
				gl::STATIC_DRAW
			);
			gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, m.ebo);
			gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
				(elements.len() * size_of::<u16>()) as isize,
				elements.as_ptr() as *const _,
				gl::STATIC_DRAW
			);
		}
		m
	}

	pub fn getTransformable(&mut self) -> &mut Transformable3D
	{
		&mut self.ts
	}
}

impl Drawable for Mesh
{
	fn draw(&mut self, cam: &mut super::Camera::Camera)
	{
		cam.shaderUse("mesh");
		cam.bindVAO(self.vao);
		cam.shaderMat4("model", self.ts.getMatrix());
		cam.shaderMat4("normalsMatrix", self.ts.getInvTrans());
		cam.shaderBool("hasMaterial", self.texture != 0);
		unsafe
		{
			if self.texture != 0
			{
				gl::ActiveTexture(gl::TEXTURE0);
				gl::BindTexture(gl::TEXTURE_2D, self.texture);
				cam.shaderInt("texture", 0);
			}
			gl::DrawElements(
				gl::TRIANGLES, self.elements,
				gl::UNSIGNED_SHORT, 0 as *const _
			);
		}
	}
}

impl Drop for Mesh
{
	fn drop(&mut self)
	{
		unsafe
		{
			if self.vao != 0 { gl::DeleteVertexArrays(1, &mut self.vao); }
			if self.vbo != 0 { gl::DeleteBuffers(1, &mut self.vbo); }
			if self.ebo != 0 { gl::DeleteBuffers(1, &mut self.ebo); }
		}
	}
}