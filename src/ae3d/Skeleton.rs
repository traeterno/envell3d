use std::{collections::HashMap};

use crate::ae3d::{glTF::{self, Node, GLTF}, Camera::Camera, Window::Window};

#[derive(Default, Debug)]
pub struct Bone
{
	pub ts: glam::Mat4,
	pub children: Vec<Bone>,
	pub name: String,
	pub id: usize,
	pub angle: glam::Mat4,
	pub pos: glam::Mat4,
	pub length: f32
}

impl Bone
{
	pub fn parse(id: usize, nodes: &HashMap<usize, Node>) -> Self
	{
		let n = nodes.get(&id).unwrap();
		let mut b = Bone::default();
		b.name = n.name.clone();
		b.id = id;
		b.angle = glam::Mat4::from_quat(n.rotation);
		b.pos = glam::Mat4::from_translation(n.translation);
		for &c in &n.childrenID
		{
			b.children.push(Bone::parse(c, nodes));
		}
		b.length =
			if n.childrenID.is_empty() { 1.0 }
			else { nodes.get(&n.childrenID[0]).unwrap().translation.length() };
		b
	}

	pub fn update(&mut self, ts: &HashMap<usize, glam::Quat>, parent: glam::Mat4) ->
		Vec<(usize, glam::Mat4)>
	{
		let offset = glam::Mat4::from_translation(glam::Vec3::Y * self.length);
		self.ts = parent * (
			if let Some(&q) = ts.get(&self.id) { glam::Mat4::from_quat(q) }
			else { self.angle }
		);

		let mut out = vec![(self.id, self.ts)];
		for b in &mut self.children
		{
			out.append(&mut b.update(ts, self.ts * offset));
		}
		out
	}
}

#[derive(Default, Debug)]
pub enum Interpolation
{
	#[default] Step,
	Linear,
	CubicSpline
}

impl Interpolation
{
	fn build(s: &String) -> Self
	{
		if s == "LINEAR" { return Self::Linear; }
		if s == "CUBICSPLINE" { return Self::CubicSpline; }
		return Self::Step;
	}

	fn apply(&self, a: f32, b: f32, t: f32) -> f32
	{
		let x = match self
		{
			Self::Step => 0.0,
			Self::Linear => t,
			Self::CubicSpline =>
				if t < 0.5 { 4.0 * t.powi(3) }
				else { 1.0 - (-2.0 * t + 2.0).powi(3) / 2.0 }
		};
		a * (1.0 - x) + b * x
	}
}

#[derive(Default, Debug)]
pub struct Keyframe
{
	timestamp: f32,
	rotation: glam::Quat,
	func: Interpolation
}

#[derive(Default, Debug)]
pub struct Animation
{
	frames: HashMap<usize, Vec<Keyframe>>,
	currentTime: f32
}

impl Animation
{
	pub fn fromGLTF(gltf: &GLTF, base: &glTF::Animation) -> (String, Self)
	{
		let mut a = Self::default();

		let mut samplers = vec![];
		
		for s in &base.samplers
		{
			let iacc = &gltf.accessors[s.input];
			let oacc = &gltf.accessors[s.output];
			let iv = &gltf.bufferViews[iacc.bufferView];
			let ov = &gltf.bufferViews[oacc.bufferView];

			let ib = &gltf.buffers[iv.buffer];
			let ob = &gltf.buffers[ov.buffer];

			let mut timestamps: Vec<f32> = vec![];

			for i in 0..iacc.count
			{
				let i = iv.byteOffset + 4 * i;
				timestamps.push(f32::from_le_bytes([
					ib[i], ib[i + 1], ib[i + 2], ib[i + 3]
				]));
			}

			let mut out: Vec<glam::Vec4> = vec![];

			for i in 0..oacc.count
			{
				let i = ov.byteOffset + 16 * i;
				out.push(glam::vec4(
					f32::from_le_bytes([ob[i+00], ob[i+01], ob[i+02], ob[i+03]]),
					f32::from_le_bytes([ob[i+04], ob[i+05], ob[i+06], ob[i+07]]),
					f32::from_le_bytes([ob[i+08], ob[i+09], ob[i+10], ob[i+11]]),
					f32::from_le_bytes([ob[i+12], ob[i+13], ob[i+14], ob[i+15]])
				));
			}

			let mut s = vec![];

			for i in 0..timestamps.len()
			{
				s.push((timestamps[i], out[i * 3 + 1]));
			}
			samplers.push(s);
		}

		for c in &base.channels
		{
			let mut timeline: Vec<Keyframe> = vec![];
			let s = &samplers[c.sampler];

			for (ts, data) in s
			{
				timeline.push(Keyframe {
					timestamp: *ts,
					rotation: glam::Quat::from_vec4(*data),
					func: Interpolation::build(&base.samplers[c.sampler].interpolation)
				});
			}

			a.frames.insert(c.node, timeline);
		}

		(base.name.clone(), a)
	}

	pub fn progress(&mut self) -> HashMap<usize, glam::Quat>
	{
		let mut ts = HashMap::new();

		self.currentTime += Window::getDeltaTime();
		self.currentTime = self.currentTime.fract();

		for (&node, timeline) in &self.frames
		{
			if timeline.len() == 1
			{
				ts.insert(node, timeline[0].rotation);
				continue;
			}
			let mut i = 0;
			while self.currentTime >= timeline[i].timestamp { i += 1; }
			let f1 = &timeline[i - 1];
			let f2 = &timeline[i];
			let t = (self.currentTime - f1.timestamp) / (f2.timestamp - f1.timestamp);
			let r1 = f1.rotation;
			let r2 = f2.rotation;
			ts.insert(node, glam::Quat::from_vec4(glam::vec4(
				f1.func.apply(r1.x, r2.x, t),
				f1.func.apply(r1.y, r2.y, t),
				f1.func.apply(r1.z, r2.z, t),
				f1.func.apply(r1.w, r2.w, t)
			)));
		}
		
		ts
	}
}

#[derive(Default, Debug)]
pub struct Skeleton
{
	root: Bone,
	joints: Vec<glam::Mat4>,
	anims: HashMap<String, Animation>,
	currentAnim: String,
	inverseBind: Vec<glam::Mat4>
}

impl Skeleton
{
	pub fn fromGLTF(gltf: &GLTF, skeleton: usize) -> Self
	{
		let info = &gltf.skins[skeleton];
		
		let acc = &gltf.accessors[info.matrices];
		let bv = &gltf.bufferViews[acc.bufferView];
		let b = &gltf.buffers[bv.buffer];

		let mut f = vec![];

		for i in 0..(bv.byteLength / 4)
		{
			let i = bv.byteOffset + i * 4;
			f.push(f32::from_le_bytes([b[i], b[i+1], b[i+2], b[i+3]]));
		}

		let mut inverseBindMatrices = vec![];

		for i in 0..acc.count
		{
			let i = i * 16;
			inverseBindMatrices.push(glam::mat4(
				glam::vec4(f[i+00], f[i+01], f[i+02], f[i+03]),
				glam::vec4(f[i+04], f[i+05], f[i+06], f[i+07]),
				glam::vec4(f[i+08], f[i+09], f[i+10], f[i+11]),
				glam::vec4(f[i+12], f[i+13], f[i+14], f[i+15]),
			));
		}

		let mut nodes: HashMap<usize, Node> = HashMap::new();
		let mut inherit: HashMap<usize, bool> = HashMap::new();

		let mut s = Skeleton::default();
		s.inverseBind = inverseBindMatrices;
		
		for &j in &info.jointsID
		{
			nodes.insert(j, gltf.nodes[j].clone());
			inherit.insert(j, false);
		}
		
		for (_, node) in &nodes
		{
			for &c in &node.childrenID
			{
				inherit.insert(c, true);
			}
		}

		for (id, &inherited) in &inherit
		{
			if !inherited
			{
				s.root = Bone::parse(*id, &nodes);
			}
		}

		for a in &gltf.animations
		{
			let (name, anim) =
				Animation::fromGLTF(gltf, a);
			s.anims.insert(name, anim);
		}

		for x in &s.inverseBind
		{
			s.joints.push(x.inverse());
		}
		
		s
	}

	pub fn setAnimation(&mut self, anim: String)
	{
		if anim != self.currentAnim
		{
			if let Some(a) = self.anims.get_mut(&self.currentAnim)
			{
				a.currentTime = 0.0;
			}
			self.currentAnim = anim;
			if let Some(a) = self.anims.get_mut(&self.currentAnim)
			{
				a.currentTime = 0.0;
			}
		}
	}

	pub fn update(&mut self, cam: &mut Camera)
	{
		cam.shaderUse("mesh");
		let ts =
			if let Some(a) = self.anims.get_mut(&self.currentAnim) { a.progress() }
			else { HashMap::new() };

		cam.shaderMat4Array("joints", &{
			let x = self.root.update(
				&ts, self.root.pos
			);
			x.iter().map(|x| x.1).collect()
		});

		cam.shaderMat4Array("bind", &self.joints);
		cam.shaderMat4Array("invBind", &self.inverseBind);
		cam.shaderInt("jc", self.joints.len() as i32);
	}
}