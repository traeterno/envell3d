use std::collections::HashMap;

use crate::ae3d::Transformable::Orientation;

use super::Window::Window;

pub trait Drawable
{
	fn draw(&mut self, cam: &mut Camera);
}

pub struct Camera
{
	offscreen: (u32, u32, u32),
	generic: (u32, u32),
	currentVAO: u32,
	currentShader: u32,
	shaders: HashMap<String, u32>,
	fov: f32,
	proj: glam::Mat4,
	scaler: i32,

	orientation: Orientation,
	distance: f32,
	pos: glam::Vec3,
	view: glam::Mat4,
	updateView: bool
}

impl Camera
{
	pub fn new() -> Self
	{
		Self
		{
			offscreen: (0, 0, 0),
			generic: (0, 0),
			currentVAO: 0,
			currentShader: 0,
			shaders: HashMap::new(),
			fov: 90.0,
			proj: glam::Mat4::IDENTITY,
			scaler: 1,
			orientation: Orientation::default(),
			distance: 0.0,
			pos: glam::Vec3::ZERO,
			view: glam::Mat4::IDENTITY,
			updateView: false
		}
	}

	pub fn load(&mut self)
	{
		unsafe
		{
			gl::GenFramebuffers(1, &mut self.offscreen.0);
			gl::BindFramebuffer(gl::FRAMEBUFFER, self.offscreen.0);

			gl::GenTextures(1, &mut self.offscreen.1);
			gl::BindTexture(gl::TEXTURE_2D, self.offscreen.1);
			gl::TexParameteri(gl::TEXTURE_2D,
				gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32
			);
			gl::TexParameteri(gl::TEXTURE_2D,
				gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32
			);
			gl::FramebufferTexture2D(
				gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0,
				gl::TEXTURE_2D, self.offscreen.1, 0
			);

			gl::GenTextures(1, &mut self.offscreen.2);
			gl::BindTexture(gl::TEXTURE_2D, self.offscreen.2);
			gl::TexParameteri(gl::TEXTURE_2D,
				gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32
			);
			gl::TexParameteri(gl::TEXTURE_2D,
				gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32
			);
			gl::FramebufferTexture2D(
				gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT,
				gl::TEXTURE_2D, self.offscreen.2, 0
			);

			gl::GenVertexArrays(1, &mut self.generic.0);
			gl::GenBuffers(1, &mut self.generic.1);
			
			gl::BindVertexArray(self.generic.0);
			gl::BindBuffer(gl::ARRAY_BUFFER, self.generic.1);
			gl::EnableVertexAttribArray(0);
			gl::VertexAttribPointer(
				0, 2, gl::FLOAT, gl::FALSE,
				2 * size_of::<f32>() as i32,
				0 as _
			);
			gl::BufferData(
				gl::ARRAY_BUFFER, 8 * size_of::<f32>() as isize,
				[0.0_f32, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0].as_ptr() as _,
				gl::STATIC_DRAW
			);
		}

		self.setup();
	}

	pub fn clear(&mut self)
	{
		Window::getProfiler().restart();
		unsafe
		{
			let (w, h) = Window::getSize();
			gl::Viewport(0, 0, w / self.scaler, h / self.scaler);
			gl::BindFramebuffer(gl::FRAMEBUFFER, self.offscreen.0);
			gl::Enable(gl::DEPTH_TEST);
			gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
			gl::Finish();
		}
		Window::getProfiler().save("clear".to_string());
	}

	pub fn display(&mut self)
	{
		Window::getProfiler().restart();
		unsafe
		{
			let (w, h) = Window::getSize();
			gl::Viewport(0, 0, w, h);
			gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
			self.shaderUse("camera");
			self.shaderInt("tex", 0);
			gl::BindTexture(gl::TEXTURE_2D, self.offscreen.1);
			gl::BindVertexArray(self.generic.0);
			gl::Disable(gl::DEPTH_TEST);
			gl::DrawArrays(gl::QUADS, 0, 4);
			gl::Finish();
		}
		Window::getProfiler().save("render".to_string());
	}

	pub fn draw(&mut self, obj: &mut impl Drawable)
	{
		if self.updateView
		{
			self.updateView = false;
			let d = self.orientation.getDirection();
			self.view = glam::Mat4::look_at_rh(
				self.pos - d * self.distance,
				self.pos + d * (if self.distance == 0.0 { 1.0 } else { 0.0 }),
				glam::Vec3::Y
			);
		}
		self.shaderMat4("worldProj", self.proj);
		self.shaderMat4("view", self.view);
		obj.draw(self);
	}

	pub fn bindVAO(&mut self, vao: u32)
	{
		if self.currentVAO == vao { return; }
		self.currentVAO = vao;
		unsafe { gl::BindVertexArray(vao); }
	}

	pub fn setFOV(&mut self, fov: f32)
	{
		self.fov = fov;
		self.setup();
	}

	pub fn getFOV(&self) -> f32 { self.fov }

	pub fn setScaler(&mut self, scaler: i32)
	{
		self.scaler = scaler;
		self.setup();
	}

	pub fn getScaler(&self) -> i32 { self.scaler }

	pub fn setDistance(&mut self, dist: f32)
	{
		self.distance = dist;
		self.updateView = true;
	}

	pub fn getDistance(&self) -> f32 { self.distance }

	pub fn setPosition(&mut self, pos: glam::Vec3)
	{
		self.pos = pos;
		self.updateView = true;
	}

	pub fn translate(&mut self, pos: glam::Vec3)
	{
		self.pos += pos;
		self.updateView = true;
	}

	pub fn getPosition(&self) -> glam::Vec3 { self.pos }

	pub fn setRotation(&mut self, angle: glam::Vec3)
	{
		self.orientation.set(angle);
		self.updateView = true;
	}

	pub fn rotate(&mut self, angle: glam::Vec3)
	{
		self.orientation.add(angle);
		self.updateView = true;
	}

	pub fn getOrientation(&self) -> &Orientation { &self.orientation }

	pub fn setup(&mut self)
	{
		let (w, h) = Window::getSize();
		self.proj = glam::Mat4::perspective_rh_gl(
			self.fov.to_radians(),
			w as f32 / h as f32,
			0.01, 2000.0
		);

		let w = w / self.scaler;
		let h = h / self.scaler;
		
		unsafe
		{
			gl::BindTexture(gl::TEXTURE_2D, self.offscreen.1);
			gl::TexImage2D(
				gl::TEXTURE_2D, 0, gl::RGB as i32,
				w, h, 0, gl::RGB,
				gl::UNSIGNED_BYTE, 0 as _
			);
			gl::BindTexture(gl::TEXTURE_2D, self.offscreen.2);
			gl::TexImage2D(
				gl::TEXTURE_2D, 0, gl::DEPTH24_STENCIL8 as i32,
				w, h, 0,
				gl::DEPTH_STENCIL, gl::UNSIGNED_INT_24_8, 0 as _
			);
		}
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

	pub fn shaderBool(&self, name: &str, value: bool)
	{
		let cn = std::ffi::CString::new(name).unwrap();
		unsafe
		{
			gl::Uniform1i(
				gl::GetUniformLocation(
					self.currentShader, cn.as_ptr()
				), if value { 1 } else { 0 }
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

	pub fn shaderMat4Array(&self, name: &str, value: &Vec<glam::Mat4>)
	{
		let cn = std::ffi::CString::new(name).unwrap();
		let mut v = vec![];
		for x in value { v.append(&mut x.to_cols_array().to_vec()); }
		unsafe
		{
			gl::UniformMatrix4fv(
				gl::GetUniformLocation(
					self.currentShader, cn.as_ptr()
				), value.len() as i32, gl::FALSE,
				v.as_ptr()
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