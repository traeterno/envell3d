use std::collections::HashMap;

use super::{Transformable::Transformable3D, Window::Window};

pub trait Drawable
{
	fn draw(&mut self, cam: &mut Camera);
}

pub struct Camera
{
	ts: Transformable3D,
	framebuffer: u32,
	stencil: u32,
	offscreen: u32,
	genericVAO: u32, genericVBO: u32,
	uiProj: glam::Mat4,
	worldProj: glam::Mat4,
	useTS: bool,
	currentVAO: u32,
	currentShader: u32,
	shaders: HashMap<String, u32>,
	fov: f32
}

impl Camera
{
	pub fn new() -> Self
	{
		Self
		{
			ts: Transformable3D::new(),
			framebuffer: 0,
			stencil: 0,
			offscreen: 0,
			genericVAO: 0, genericVBO: 0,
			uiProj: glam::Mat4::IDENTITY,
			worldProj: glam::Mat4::IDENTITY,
			useTS: false,
			currentVAO: 0,
			currentShader: 0,
			shaders: HashMap::new(),
			fov: 90.0
		}
	}

	pub fn load(&mut self)
	{
		let (w, h) = Window::getSize();

		unsafe
		{
			gl::GenFramebuffers(1, &mut self.framebuffer);
			gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);

			gl::GenTextures(1, &mut self.offscreen);
			gl::BindTexture(gl::TEXTURE_2D, self.offscreen);
			gl::TexImage2D(
				gl::TEXTURE_2D, 0, gl::RGB as i32,
				w, h, 0,
				gl::RGB, gl::UNSIGNED_BYTE, 0 as _
			);
			gl::TexParameteri(gl::TEXTURE_2D,
				gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32
			);
			gl::TexParameteri(gl::TEXTURE_2D,
				gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32
			);

			gl::FramebufferTexture2D(
				gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0,
				gl::TEXTURE_2D, self.offscreen, 0
			);

			gl::GenTextures(1, &mut self.stencil);
			gl::BindTexture(gl::TEXTURE_2D, self.stencil);
			gl::TexImage2D(
				gl::TEXTURE_2D, 0, gl::DEPTH24_STENCIL8 as i32,
				w, h, 0,
				gl::DEPTH_STENCIL, gl::UNSIGNED_INT_24_8, 0 as _
			);

			gl::FramebufferTexture2D(
				gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT,
				gl::TEXTURE_2D, self.stencil, 0
			);
			
			gl::GenVertexArrays(1, &mut self.genericVAO);
			gl::GenBuffers(1, &mut self.genericVBO);

			gl::BindVertexArray(self.genericVAO);
			gl::BindBuffer(gl::ARRAY_BUFFER, self.genericVBO);

			gl::EnableVertexAttribArray(0);

			gl::VertexAttribPointer(
				0, 2, gl::FLOAT, gl::FALSE,
				(2 * size_of::<f32>()) as i32, 0 as _
			);

			gl::BufferData(gl::ARRAY_BUFFER,
				(8 * size_of::<f32>()) as _,
					[0.0_f32, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0].as_ptr() as _,
				gl::STATIC_DRAW
			);

			gl::ClearColor(0.5, 0.5, 0.5, 1.0);

			gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
		}

		self.setSize((w, h));
		self.setup((w, h), self.fov);
		self.toggleTransform(true);
	}

	pub fn clear(&mut self)
	{
		Window::getProfiler().restart();
		unsafe
		{
			// gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
			gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
			gl::Finish();
		}
		Window::getProfiler().save("clear".to_string());
	}

	pub fn display(&mut self)
	{
		Window::getProfiler().restart();
		// unsafe
		// {
		// 	gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
		// 	self.shaderUse("camera");
		// 	self.shaderInt("tex", 0);
		// 	gl::BindTexture(gl::TEXTURE_2D, self.offscreen);
		// 	gl::BindVertexArray(self.genericVAO);
		// 	gl::DrawArrays(gl::QUADS, 0, 4);
		// 	gl::Finish();
		// }
		Window::getProfiler().save("render".to_string());
	}

	pub fn toggleTransform(&mut self, enable: bool)
	{
		self.useTS = enable;
		let proj = if enable { self.worldProj } else { self.uiProj };
		let view = if enable {self.ts.getMatrix()} else {glam::Mat4::IDENTITY};
		for (_, &s) in &self.shaders
		{
			unsafe
			{
				gl::UseProgram(s);
			}
			self.currentShader = s;
			self.shaderMat4("projection", proj);
			self.shaderMat4("view", view);
		}
	}

	pub fn setSize(&mut self, s: (i32, i32))
	{
		self.setup(s, self.fov);
		self.uiProj = glam::Mat4::orthographic_rh_gl(
			0.0, s.0 as f32,
			s.1 as f32, 0.0,
			-1.0, 1.0
		);
		// unsafe
		// {
		// 	gl::BindTexture(gl::TEXTURE_2D, self.offscreen);
		// 	gl::TexImage2D(
		// 		gl::TEXTURE_2D, 0, gl::RGB as i32,
		// 		s.0, s.1, 0, gl::RGB,
		// 		gl::UNSIGNED_BYTE, 0 as _
		// 	);
		// 	gl::BindTexture(gl::TEXTURE_2D, self.stencil);
		// 	gl::TexImage2D(
		// 		gl::TEXTURE_2D, 0, gl::DEPTH24_STENCIL8 as i32,
		// 		s.0, s.1, 0,
		// 		gl::DEPTH_STENCIL, gl::UNSIGNED_INT_24_8, 0 as _
		// 	);
		// }

	}

	pub fn draw(&mut self, obj: &mut impl Drawable)
	{
		obj.draw(self);
	}

	pub fn genericVAO(&mut self)
	{
		self.bindVAO(self.genericVAO);
	}

	pub fn bindVAO(&mut self, vao: u32)
	{
		if self.currentVAO == vao { return; }
		self.currentVAO = vao;
		unsafe { gl::BindVertexArray(vao); }
	}

	pub fn getTransformable(&mut self) -> &mut Transformable3D
	{
		&mut self.ts
	}

	pub fn drawRect(&mut self, ts: glam::Mat4, clr: glam::Vec4)
	{
		self.shaderUse("shape");
		self.shaderMat4("model", ts);
		self.shaderVec4("clr", clr);
		self.shaderInt("mode", 0);
		self.genericVAO();
		unsafe { gl::DrawArrays(gl::QUADS, 0, 4); }
	}

	pub fn drawLine(&mut self, p1: glam::Vec2, p2: glam::Vec2, clr: glam::Vec4)
	{
		self.shaderUse("shape");
		self.shaderVec4("clr", clr);
		self.shaderVec2("p1", p1);
		self.shaderVec2("p2", p2);
		self.shaderInt("mode", 1);
		self.genericVAO();
		unsafe { gl::DrawArrays(gl::LINES, 0, 2); }
	}

	pub fn setup(&mut self, s: (i32, i32), fov: f32)
	{
		self.fov = fov;
		self.worldProj = glam::Mat4::perspective_rh_gl(
			fov.to_radians(),
			s.0 as f32 / s.1 as f32,
			// 1.0,
			0.01, 2000.0
		);
	}

	pub fn lookAt(&mut self, p: glam::Vec3)
	{
		self.ts.lookAt(p);
	}

	pub fn shaderUse(&mut self, shader: &str)
	{
		if let Some(&s) = self.shaders.get(&shader.to_string())
		{
			if s != self.currentShader
			{
				unsafe
				{
					self.currentShader = s;
					gl::UseProgram(s);
				}
			}
			return;
		}
		let vertex = Self::shaderLoad(shader, gl::VERTEX_SHADER);
		let fragment = Self::shaderLoad(shader, gl::FRAGMENT_SHADER);
		unsafe
		{
			let program = gl::CreateProgram();
			if vertex != 0 { gl::AttachShader(program, vertex); }
			if fragment != 0 { gl::AttachShader(program, fragment); }
			gl::LinkProgram(program);
			let mut status = 0;
			gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
			if status == 0
			{
				let mut infoLog = [0; 512];
				let mut written = 0;
				gl::GetProgramInfoLog(
					program, 512,
					&mut written, infoLog.as_mut_ptr()
				);
				println!("Failed to link shader:\n{}", String::from_raw_parts(
					infoLog.as_mut_ptr() as *mut u8, written as usize, 512
				));
			}
			if vertex != 0
			{
				gl::DetachShader(program, vertex);
				gl::DeleteShader(vertex);
			}
			if fragment != 0
			{
				gl::DetachShader(program, fragment);
				gl::DeleteShader(fragment);
			}

			self.shaders.insert(shader.to_string(), program);
			gl::UseProgram(program);
			self.currentShader = program;
		}
	}

	fn shaderLoad(path: &str, t: gl::types::GLenum) -> u32
	{
		let ext = if t == gl::VERTEX_SHADER { ".vert" } else { ".frag" };
		let code = std::ffi::CString::new(std::fs::read_to_string(
			String::from("res/shaders/") + path + ext
		).expect(&format!("Failed to load '{path}{ext}' shader."))).unwrap();
		unsafe
		{
			let shader = gl::CreateShader(t);
			gl::ShaderSource(shader, 1, &code.as_ptr(), 0 as _);
			gl::CompileShader(shader);
			let mut status = 0;
			gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
			if status == 0
			{
				let mut len = 0;
				gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
				let mut buf: Vec<u8> = Vec::with_capacity(len as usize + 1);
				buf.extend([b' '].iter().cycle().take(len as usize));
				let error = std::ffi::CString::from_vec_unchecked(buf);
				gl::GetShaderInfoLog(
					shader, len,
					std::ptr::null_mut(), error.as_ptr() as *mut i8
				);
				println!("Failed to compile shader {path}{ext}:\n{}", error.to_str().unwrap());
			}
			shader
		}
	}

	pub fn shaderInt(&self, name: &str, value: i32)
	{
		let cn = std::ffi::CString::new(name).unwrap();
		unsafe
		{
			gl::Uniform1i(
				gl::GetUniformLocation(
					self.currentShader, cn.as_ptr()
				), value
			);
		}
	}

	pub fn shaderMat4(&self, name: &str, value: glam::Mat4)
	{
		let cn = std::ffi::CString::new(name).unwrap();
		unsafe
		{
			gl::UniformMatrix4fv(
				gl::GetUniformLocation(
					self.currentShader, cn.as_ptr()
				), 1, gl::FALSE,
				value.to_cols_array().as_ptr()
			);
		}
	}

	pub fn shaderVec2(&self, name: &str, value: glam::Vec2)
	{
		let cn = std::ffi::CString::new(name).unwrap();
		unsafe
		{
			gl::Uniform2f(
				gl::GetUniformLocation(
					self.currentShader, cn.as_ptr()
				), value.x, value.y
			);
		}
	}

	pub fn shaderVec3(&self, name: &str, value: glam::Vec3)
	{
		let cn = std::ffi::CString::new(name).unwrap();
		unsafe
		{
			gl::Uniform3f(
				gl::GetUniformLocation(
					self.currentShader, cn.as_ptr()
				), value.x, value.y, value.z
			);
		}
	}

	pub fn shaderVec4(&self, name: &str, value: glam::Vec4)
	{
		let cn = std::ffi::CString::new(name).unwrap();
		unsafe
		{
			gl::Uniform4f(
				gl::GetUniformLocation(
					self.currentShader, cn.as_ptr()
				), value.x, value.y, value.z, value.w
			);
		}
	}

	pub fn clearShaders(&mut self)
	{
		for (_, &s) in &self.shaders
		{
			unsafe { gl::DeleteProgram(s); }
		}
		self.shaders.clear();
	}
}