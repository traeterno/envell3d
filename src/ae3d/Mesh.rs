use crate::ae3d::{Camera::Drawable, Transformable::Transformable3D};

pub struct Mesh
{
	ts: Transformable3D,
	vertices: Vec<f32>,
	vao: u32,
	vbo: u32,
}

impl Mesh
{
	pub fn new() -> Self
	{
		Self
		{
			ts: Transformable3D::new(),
			vertices: vec![],
			vao: 0,
			vbo: 0,
		}
	}

	pub fn load(&mut self, path: String)
	{
		unsafe
		{
			if self.vao != 0 { gl::DeleteVertexArrays(1, &mut self.vao); }
			if self.vbo != 0 { gl::DeleteBuffers(1, &mut self.vbo); }
		}

		self.vertices.clear();
		
		let mut vertices = vec![];
		let mut normals = vec![];

		if let Ok(f) = std::fs::read_to_string(path)
		{
			for l in f.lines()
			{
				if l.starts_with("vn")
				{
					for x in l.split(" ").filter(|x| x.parse::<f32>().is_ok())
					{
						normals.push(x.parse::<f32>().unwrap());
					}
				}
				else if l.starts_with("v")
				{
					for x in l.split(" ").filter(|x| x.parse::<f32>().is_ok())
					{
						vertices.push(x.parse::<f32>().unwrap());
					}
				}
				else if l.starts_with("f")
				{
					for v in l.split(" ").filter(|x| x.contains("/"))
					{
						let vtn: Vec<&str> = v.split("/").collect();
						let v = vtn[0].parse::<usize>().unwrap();
						// let t = vtn[1].parse::<usize>().unwrap_or(0);
						let n = vtn[2].parse::<usize>().unwrap_or(0);
						if v > 0
						{
							let v = (v - 1) * 3;
							self.vertices.push(vertices[v]);
							self.vertices.push(vertices[v + 1]);
							self.vertices.push(vertices[v + 2]);
						}
						else { self.vertices.append(&mut vec![0.0, 0.0, 0.0]); }
						if n > 0
						{
							let n = (n - 1) * 3;
							self.vertices.push(normals[n]);
							self.vertices.push(normals[n + 1]);
							self.vertices.push(normals[n + 2]);
						}
						else { self.vertices.append(&mut vec![0.0, 0.0, 0.0]); }
					}
				}
			}
		}
		
		let mut vao = 0;
		let mut vbo = 0;
		unsafe
		{
			gl::GenVertexArrays(1, &mut vao); self.vao = vao;
			gl::GenBuffers(1, &mut vbo); self.vbo = vbo;

			gl::BindVertexArray(vao);
			gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
			gl::BufferData(gl::ARRAY_BUFFER,
				(self.vertices.len() * size_of::<f32>()) as isize,
				self.vertices.as_ptr() as *const _,
				gl::STATIC_DRAW
			);

			gl::EnableVertexAttribArray(0);

			gl::VertexAttribPointer(
				0, 3, gl::FLOAT, gl::FALSE,
				(6 * size_of::<f32>()) as i32, 0 as _
			);

			gl::EnableVertexAttribArray(1);

			gl::VertexAttribPointer(
				1, 3, gl::FLOAT, gl::FALSE,
				(6 * size_of::<f32>()) as i32,
				(3 * size_of::<f32>()) as _
			);
		}
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
		unsafe
		{
			gl::DrawArrays(gl::TRIANGLES, 0, self.vertices.len() as i32 / 3);
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
		}
	}
}