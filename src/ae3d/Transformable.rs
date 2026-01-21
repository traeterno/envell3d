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
pub struct Transformable3D
{
	position: glam::Vec3,
	orientation: Orientation,
	scale: f32,

	model: glam::Mat4,
	invTrans: glam::Mat4,
	reloadModel: bool
}

impl Transformable3D
{
	pub fn new() -> Self
	{
		Self
		{
			position: glam::Vec3::ZERO,
			orientation: Orientation::default(),
			scale: 1.0,
			
			model: glam::Mat4::IDENTITY,
			invTrans: glam::Mat4::IDENTITY,
			reloadModel: true
		}
	}

	fn update(&mut self)
	{
		self.model =
			glam::Mat4::from_translation(self.position) *
			self.orientation.getMatrix() *
			glam::Mat4::from_scale(glam::Vec3::splat(self.scale));
		
		self.invTrans = self.model.inverse().transpose();
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

	pub fn getOrientation(&mut self) -> &mut Orientation
	{
		&mut self.orientation
	}

	pub fn setScale(&mut self, scale: f32)
	{
		self.scale = scale;
		self.reloadModel = true;
	}

	pub fn scale(&mut self, factor: f32)
	{
		self.scale *= factor;
		self.reloadModel = true;
	}

	pub fn getScale(&self) -> f32 { self.scale }
}

#[derive(Debug, Clone)]
pub struct Orientation
{
	quat: glam::Quat,
	angle: glam::Vec3,
	direction: glam::Vec3,
	right: glam::Vec3,
	up: glam::Vec3,
	model: glam::Mat4,
	reload: bool
}

impl Default for Orientation
{
	fn default() -> Self
	{
		Self
		{
			quat: glam::Quat::IDENTITY,
			angle: glam::Vec3::ZERO,
			direction: glam::Vec3::Z,
			right: glam::Vec3::X,
			up: glam::Vec3::Y,
			model: glam::Mat4::IDENTITY,
			reload: false
		}
	}
}

impl Orientation
{
	pub fn set(&mut self, angle: glam::Vec3)
	{
		self.angle = angle;
		self.angle.x = self.angle.x % 360.0;
		self.angle.y = self.angle.y % 360.0;
		self.quat = glam::Quat::from_rotation_y(angle.x.to_radians());
		self.right = self.quat.mul_vec3(glam::Vec3::X);
		let pitch = glam::Quat::from_axis_angle(self.right, angle.y.to_radians());
		self.quat = pitch * self.quat;
		self.direction = self.quat.mul_vec3(glam::Vec3::Z);
		let roll = glam::Quat::from_axis_angle(self.direction, angle.z.to_radians());
		self.quat = roll * self.quat;
		self.reload = true;
	}

	pub fn add(&mut self, angle: glam::Vec3)
	{
		self.set(self.angle + angle);
	}

	pub fn getMatrix(&mut self) -> glam::Mat4
	{
		if self.reload
		{
			self.model = glam::Mat4::from_quat(self.quat);
			self.reload = false;
		}

		self.model
	}

	pub fn getAngle(&self) -> glam::Vec3 { self.angle }
	pub fn getDirection(&self) -> glam::Vec3 { self.direction }
	pub fn getRight(&self) -> glam::Vec3 { self.right }
	pub fn getUp(&self) -> glam::Vec3 { self.up }
}