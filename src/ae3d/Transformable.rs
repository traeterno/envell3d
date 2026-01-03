#[derive(Debug, Clone)]
pub struct Transformable2D
{
	position: glam::Vec2,
	rotation: f32,
	scale: glam::Vec2,
	origin: glam::Vec2,
	model: glam::Mat4,
	reloadModel: bool
}

impl Transformable2D
{
	pub fn new() -> Self
	{
		Self
		{
			position: glam::Vec2::ZERO,
			rotation: 0.0,
			scale: glam::Vec2::ONE,
			origin: glam::Vec2::ZERO,
			model: glam::Mat4::IDENTITY,
			reloadModel: true
		}
	}

	fn update(&mut self)
	{
		self.model =
			glam::Mat4::from_translation(glam::vec3(self.position.x, self.position.y, 0.0))
			.mul_mat4(&glam::Mat4::from_rotation_z(self.rotation.to_radians()))
			.mul_mat4(&glam::Mat4::from_scale(glam::vec3(self.scale.x, self.scale.y, 1.0)))
			.mul_mat4(&glam::Mat4::from_translation(-glam::vec3(self.origin.x, self.origin.y, 0.0)));
	}

	pub fn getMatrix(&mut self) -> glam::Mat4
	{
		if self.reloadModel { self.update(); }

		self.model
	}

	pub fn setPosition(&mut self, pos: glam::Vec2) { self.position = pos; self.reloadModel = true; }
	pub fn translate(&mut self, delta: glam::Vec2) { self.position += delta; self.reloadModel = true; }
	pub fn getPosition(&mut self) -> glam::Vec2 { self.position }

	pub fn setRotation(&mut self, angle: f32) { self.rotation = angle; self.reloadModel = true; }
	pub fn rotate(&mut self, delta: f32) { self.rotation += delta; self.reloadModel = true; }
	pub fn getRotation(&mut self) -> f32{ self.rotation }

	pub fn setScale(&mut self, scale: glam::Vec2) { self.scale = scale; self.reloadModel = true; }
	pub fn scale(&mut self, delta: glam::Vec2) { self.scale *= delta; self.reloadModel = true; }
	pub fn getScale(&mut self) -> glam::Vec2 { self.scale }

	pub fn setOrigin(&mut self, origin: glam::Vec2) { self.origin = origin; self.reloadModel = true; }
	pub fn getOrigin(&mut self) -> glam::Vec2 { self.origin }

	pub fn quick(pos: glam::Vec2, angle: f32, scale: glam::Vec2, origin: glam::Vec2) -> glam::Mat4
	{
		glam::Mat4::from_translation(glam::vec3(pos.x, pos.y, 0.0))
			.mul_mat4(&glam::Mat4::from_rotation_z(angle.to_radians()))
			.mul_mat4(&glam::Mat4::from_scale(glam::vec3(scale.x, scale.y, 1.0)))
			.mul_mat4(&glam::Mat4::from_translation(-glam::vec3(origin.x, origin.y, 0.0)))
	}
}

#[derive(Default, Debug, Clone)]
pub enum RotationMode
{
	#[default] Euler,
	LookAtFP,
	LookAtTP(f32)
}

#[derive(Default, Debug, Clone)]
pub struct Transformable3D
{
	position: glam::Vec3,
	direction: glam::Vec3,
	front: glam::Vec2,
	angle: glam::Vec2,

	model: glam::Mat4,
	invTrans: glam::Mat4,
	reloadModel: bool,
	rotationMode: RotationMode
}

impl Transformable3D
{
	pub fn new() -> Self
	{
		Self
		{
			position: glam::Vec3::ZERO,
			direction: glam::Vec3::Z,
			front: glam::Vec2::Y,
			angle: glam::Vec2::ZERO,
			
			model: glam::Mat4::IDENTITY,
			invTrans: glam::Mat4::IDENTITY,
			reloadModel: true,
			rotationMode: RotationMode::Euler
		}
	}

	fn update(&mut self)
	{
		self.model = match self.rotationMode
		{
			RotationMode::Euler =>
			{
				glam::Mat4::from_translation(self.position) *
				glam::Mat4::from_rotation_y(self.angle.x.to_radians()) *
				glam::Mat4::from_rotation_x(self.angle.y.to_radians())
			}
			RotationMode::LookAtFP =>
			{
				glam::Mat4::look_at_rh(
					self.position,
					self.position + self.direction,
					glam::Vec3::Y
				)
			}
			RotationMode::LookAtTP(dist) =>
			{
				glam::Mat4::look_at_rh(
					self.position - self.direction * dist,
					self.position,
					glam::Vec3::Y
				)
			}
		};
		
		self.invTrans = self.model.inverse().transpose();
	}

	pub fn setRotationMode(&mut self, mode: RotationMode)
	{
		self.rotationMode = mode;
		self.reloadModel = true;
	}

	pub fn getMatrix(&mut self) -> glam::Mat4
	{
		if self.reloadModel { self.update(); }

		self.model
	}

	pub fn getInvTrans(&mut self) -> glam::Mat4
	{
		if self.reloadModel { self.update(); }

		self.invTrans
	}

	pub fn setPosition(&mut self, pos: glam::Vec3)
	{
		self.position = pos; self.reloadModel = true;
	}
	pub fn translate(&mut self, delta: glam::Vec3)
	{
		self.position += delta; self.reloadModel = true;
	}
	pub fn getPosition(&self) -> glam::Vec3 { self.position }

	pub fn setRotation(&mut self, angle: glam::Vec2)
	{
		self.angle.x = angle.x % 360.0;
		self.angle.y = angle.y.clamp(-89.0, 89.0);
		let yaw = self.angle.x.to_radians();
		let pitch = self.angle.y.to_radians();

		self.front = glam::vec2(yaw.cos(), yaw.sin());

		self.direction = glam::vec3(
			self.front.x * pitch.cos(),
			pitch.sin(),
			self.front.y * pitch.cos()
		);

		self.reloadModel = true;
	}

	pub fn rotate(&mut self, angle: glam::Vec2)
	{
		self.setRotation(self.angle + angle);
	}

	pub fn getDirection(&self) -> glam::Vec3 { self.direction }
	pub fn getFront(&self) -> glam::Vec2 { self.front }
	pub fn getRotation(&self) -> glam::Vec2 { self.angle }
}