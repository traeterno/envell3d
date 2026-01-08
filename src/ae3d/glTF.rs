use std::collections::HashMap;

use crate::ae3d::Window::Window;

#[derive(Default, Debug)]
pub struct BufferView
{
	pub buffer: usize,
	pub byteLength: usize,
	pub byteOffset: usize,
	pub target: u16
}

#[derive(Default, Debug)]
pub struct Accessor
{
	pub bufferView: usize,
	pub componentType: u16,
	pub count: usize,
	pub valueType: String
}

pub type Primitive = HashMap<String, usize>;

#[derive(Default, Debug, Clone)]
pub struct Mesh
{
	pub name: String,
	pub primitives: Vec<Primitive>
}

#[derive(Default, Debug, Clone)]
pub struct Node
{
	pub name: String,
	pub translation: glam::Vec3,
	pub rotation: glam::Quat,
	pub scale: glam::Vec3,
	pub childrenID: Vec<usize>,
	pub meshID: usize,
	pub skin: usize
}

#[derive(Default, Debug, Clone)]
pub struct Skin
{
	pub matrices: usize,
	pub jointsID: Vec<usize>,
	pub name: String
}

#[derive(Default, Debug, Clone)]
pub struct AnimationSampler
{
	pub input: usize,
	pub interpolation: String,
	pub output: usize
}

#[derive(Default, Debug, Clone)]
pub struct AnimationChannel
{
	pub sampler: usize,
	pub node: usize,
	pub path: String
}

#[derive(Default, Debug, Clone)]
pub struct Animation
{
	pub name: String,
	pub samplers: Vec<AnimationSampler>,
	pub channels: Vec<AnimationChannel>,
	pub duration: f32
}

#[derive(Default, Debug)]
pub struct Scene
{
	pub name: String,
	pub nodesID: Vec<usize>,
}

#[derive(Default, Debug)]
pub struct TextureSampler
{
	pub minFilter: i32,
	pub magFilter: i32
}

#[derive(Default, Debug)]
pub struct Image
{
	pub mimeType: String,
	pub name: String,
	pub uri: String
}

#[derive(Default, Debug)]
pub struct Texture
{
	pub sampler: usize,
	pub source: usize
}

#[derive(Default, Debug)]
pub struct Material
{
	pub name: String,
	pub texture: usize
}

#[derive(Default, Debug)]
pub struct GLTF
{
	pub buffers: Vec<Vec<u8>>,
	pub bufferViews: Vec<BufferView>,
	pub accessors: Vec<Accessor>,
	pub scene: usize,
	pub scenes: Vec<Scene>,
	pub nodes: Vec<Node>,
	pub meshes: Vec<Mesh>,
	pub skins: Vec<Skin>,
	pub animations: Vec<Animation>,
	pub samplers: Vec<TextureSampler>,
	pub images: Vec<Image>,
	pub textures: Vec<Texture>,
	pub materials: Vec<Material>
}

impl GLTF
{
	pub fn load(path: String) -> Self
	{
		let mut gltf = Self::default();
		let src = std::fs::read_to_string(&path);
		if src.is_err() { println!("Error READ {path}: {src:?}"); return gltf; }
		let src = json::parse(&src.unwrap());
		if src.is_err() { println!("Error PARSE {path}: {src:?}"); return gltf; }

		// TODO simplify parser

		for section in src.unwrap().entries()
		{
			if section.0 == "buffers"
			{
				for buf in section.1.members()
				{
					for var in buf.entries()
					{
						if var.0 == "uri"
						{
							let mut p: Vec<&str> = path.split("/").collect();
							p.remove(p.len() - 1);
							p.push(var.1.as_str().unwrap());
							match std::fs::read(p.join("/"))
							{
								Ok(b) => gltf.buffers.push(b),
								Err(x) => println!("Failed {path}: {x:#?}")
							}
						}
					}
				}
			}
			else if section.0 == "bufferViews"
			{
				for view in section.1.members()
				{
					let mut bv = BufferView::default();
					for var in view.entries()
					{
						if var.0 == "buffer" { bv.buffer = var.1.as_usize().unwrap(); }
						if var.0 == "byteLength" { bv.byteLength = var.1.as_usize().unwrap(); }
						if var.0 == "byteOffset" { bv.byteOffset = var.1.as_usize().unwrap(); }
						if var.0 == "target" { bv.target = var.1.as_u16().unwrap(); }
					}
					gltf.bufferViews.push(bv);
				}
			}
			else if section.0 == "accessors"
			{
				for a in section.1.members()
				{
					let mut acc = Accessor::default();
					for var in a.entries()
					{
						if var.0 == "bufferView" { acc.bufferView = var.1.as_usize().unwrap(); }
						if var.0 == "componentType" { acc.componentType = var.1.as_u16().unwrap(); }
						if var.0 == "count" { acc.count = var.1.as_usize().unwrap(); }
						if var.0 == "type" { acc.valueType = var.1.as_str().unwrap().to_string(); }
					}
					gltf.accessors.push(acc);
				}
			}
			else if section.0 == "scene" { gltf.scene = section.1.as_usize().unwrap(); }
			else if section.0 == "scenes"
			{
				for scene in section.1.members()
				{
					let mut s = Scene::default();
					for var in scene.entries()
					{
						if var.0 == "name" { s.name = var.1.as_str().unwrap().to_string(); }
						if var.0 == "nodes"
						{
							for node in var.1.members()
							{
								s.nodesID.push(node.as_usize().unwrap());
							}
						}
					}
					gltf.scenes.push(s);
				}
			}
			else if section.0 == "nodes"
			{
				for node in section.1.members()
				{
					let mut n = Node::default();
					for var in node.entries()
					{
						if var.0 == "name" { n.name = var.1.as_str().unwrap().to_string(); }
						if var.0 == "translation"
						{
							let x: Vec<f32> = var.1.members()
								.map(|x| x.as_f32().unwrap()).collect();
							n.translation = glam::vec3(x[0], x[1], x[2]);
						}
						if var.0 == "translation"
						{
							let x: Vec<f32> = var.1.members()
								.map(|x| x.as_f32().unwrap()).collect();
							n.scale = glam::vec3(x[0], x[1], x[2]);
						}
						if var.0 == "rotation"
						{
							let x: Vec<f32> = var.1.members()
								.map(|x| x.as_f32().unwrap()).collect();
							n.rotation = glam::Quat::from_vec4(
								glam::vec4(x[0], x[1], x[2], x[3])
							);
						}
						if var.0 == "children"
						{
							n.childrenID = var.1.members()
								.map(|x| x.as_usize().unwrap()).collect();
						}
						if var.0 == "mesh" { n.meshID = var.1.as_usize().unwrap(); }
						if var.0 == "skin" { n.skin = var.1.as_usize().unwrap(); }
					}
					gltf.nodes.push(n);
				}
			}
			else if section.0 == "meshes"
			{
				for mesh in section.1.members()
				{
					let mut m = Mesh::default();
					for var in mesh.entries()
					{
						if var.0 == "name" { m.name = var.1.as_str().unwrap().to_string(); }
						if var.0 == "primitives"
						{
							for primitive in var.1.members()
							{
								let mut p = Primitive::new();
								for var in primitive.entries()
								{
									if var.0 == "attributes"
									{
										for attr in var.1.entries()
										{
											p.insert(
												attr.0.to_string(),
												attr.1.as_usize().unwrap()
											);
										}
									}
									else
									{
										p.insert(
											var.0.to_string(),
											var.1.as_usize().unwrap()
										);
									}
								}
								m.primitives.push(p);
							}
						}
					}
					gltf.meshes.push(m);
				}
			}
			else if section.0 == "skins"
			{
				for skin in section.1.members()
				{
					let mut s = Skin::default();
					for var in skin.entries()
					{
						if var.0 == "inverseBindMatrices"
						{
							s.matrices = var.1.as_usize().unwrap();
						}
						if var.0 == "joints"
						{
							for j in var.1.members()
							{
								s.jointsID.push(j.as_usize().unwrap());
							}
						}
						if var.0 == "name"
						{
							s.name = var.1.as_str().unwrap().to_string();
						}
					}
					gltf.skins.push(s);
				}
			}
			else if section.0 == "animations"
			{
				for anim in section.1.members()
				{
					let mut a = Animation::default();
					a.duration = 1.0;
					for var in anim.entries()
					{
						if var.0 == "name"
						{
							a.name = var.1.as_str().unwrap().to_string();
						}
						if var.0 == "channels"
						{
							for channel in var.1.members()
							{
								let mut c = AnimationChannel::default();
								for v in channel.entries()
								{
									if v.0 == "sampler"
									{
										c.sampler = v.1.as_usize().unwrap();
									}
									if v.0 == "target"
									{
										for x in v.1.entries()
										{
											if x.0 == "node"
											{
												c.node = x.1.as_usize().unwrap();
											}
											if x.0 == "path"
											{
												c.path = x.1.as_str().unwrap().to_string();
											}
										}
									}
								}
								a.channels.push(c);
							}
						}
						if var.0 == "samplers"
						{
							for sampler in var.1.members()
							{
								let mut s = AnimationSampler::default();
								for var in sampler.entries()
								{
									if var.0 == "input"
									{
										s.input = var.1.as_usize().unwrap();
									}
									if var.0 == "output"
									{
										s.output = var.1.as_usize().unwrap();
									}
									if var.0 == "interpolation"
									{
										s.interpolation = var.1.as_str().unwrap().to_string();
									}
								}
								a.samplers.push(s);
							}
						}
						if var.0 == "extras"
						{
							for x in var.1.entries()
							{
								if x.0 == "duration"
								{
									a.duration = x.1.as_f32().unwrap();
								}
							}
						}
					}
					gltf.animations.push(a);
				}
			}
			else if section.0 == "samplers"
			{
				for sampler in section.1.members()
				{
					let mut s = TextureSampler::default();
					for var in sampler.entries()
					{
						if var.0 == "minFilter"
						{
							s.minFilter = var.1.as_i32().unwrap();
						}
						if var.0 == "magFilter"
						{
							s.magFilter = var.1.as_i32().unwrap();
						}
					}
					gltf.samplers.push(s);
				}
			}
			else if section.0 == "images"
			{
				for image in section.1.members()
				{
					let mut i = Image::default();
					for var in image.entries()
					{
						if var.0 == "mimeType"
						{
							i.mimeType = var.1.as_str().unwrap().to_string();
						}
						if var.0 == "name"
						{
							i.name = var.1.as_str().unwrap().to_string();
						}
						if var.0 == "uri"
						{
							let mut p: Vec<&str> = path.split("/").collect();
							p.remove(p.len() - 1);
							p.push(var.1.as_str().unwrap());
							i.uri = p.join("/");
						}
					}
					gltf.images.push(i);
				}
			}
			else if section.0 == "textures"
			{
				for texture in section.1.members()
				{
					let mut t = Texture::default();
					for var in texture.entries()
					{
						if var.0 == "sampler"
						{
							t.sampler = var.1.as_usize().unwrap();
						}
						if var.0 == "source"
						{
							t.source = var.1.as_usize().unwrap();
						}
					}
					gltf.textures.push(t);
				}
			}
			else if section.0 == "materials"
			{
				for material in section.1.members()
				{
					let mut m = Material::default();
					for var in material.entries()
					{
						if var.0 == "name"
						{
							m.name = var.1.as_str().unwrap().to_string();
						}
						if var.0 == "pbrMetallicRoughness"
						{
							for v in var.1.entries()
							{
								if v.0 == "baseColorTexture"
								{
									for x in v.1.entries()
									{
										if x.0 == "index"
										{
											m.texture = x.1.as_usize().unwrap();
										}
									}
								}
							}
						}
					}
					gltf.materials.push(m);
				}
			}
			else if section.0 == "asset" {}
			else { println!("{}", section.0); }
		}
		
		gltf
	}

	pub fn mesh(&self, id: usize) ->
		(Vec<f32>, Vec<f32>, Vec<u16>, Vec<f32>, Vec<f32>, u32)
	{
		let info = &self.meshes[id];
		let mut verticesID = usize::MAX;
		let mut normalsID = usize::MAX;
		let mut elementsID = usize::MAX;
		let mut jointsID = usize::MAX;
		let mut uvsID = usize::MAX;
		let mut materialID = usize::MAX;
		// let mut weightsID = 0;
		for (key, &value) in &info.primitives[0]
		{
			if key == "POSITION" { verticesID = value; }
			if key == "NORMAL" { normalsID = value; }
			if key == "indices" { elementsID = value; }
			if key == "JOINTS_0" { jointsID = value; }
			if key == "TEXCOORD_0" { uvsID = value; }
			if key == "material" { materialID = value; }
		}

		let mut vertices: Vec<f32> = vec![];
		let mut normals: Vec<f32> = vec![];
		let mut elements: Vec<u16> = vec![];
		let mut joints: Vec<f32> = vec![];
		let mut uvs: Vec<f32> = vec![];
		let mut material = 0u32;

		if verticesID != usize::MAX
		{
			let va = &self.accessors[verticesID];
			let vbv = &self.bufferViews[va.bufferView];
			let vb = &self.buffers[vbv.buffer];
			for i in 0..(va.count * 3)
			{
				let v = vbv.byteOffset + i * 4;
				vertices.push(f32::from_le_bytes([vb[v], vb[v + 1], vb[v + 2], vb[v + 3]]));
			}
		}

		if normalsID != usize::MAX
		{
			let na = &self.accessors[normalsID];
			let nbv = &self.bufferViews[na.bufferView];
			let nb = &self.buffers[nbv.buffer];
			for i in 0..(na.count * 3)
			{
				let n = nbv.byteOffset + i * 4;
				normals.push(f32::from_le_bytes([nb[n], nb[n + 1], nb[n + 2], nb[n + 3]]));
			}
		}

		if elementsID != usize::MAX
		{
			let ea = &self.accessors[elementsID];
			let ebv = &self.bufferViews[ea.bufferView];
			let eb = &self.buffers[ebv.buffer];
			for i in 0..ea.count
			{
				let e = ebv.byteOffset + i * 2;
				elements.push(u16::from_le_bytes([eb[e], eb[e + 1]]));
			}
		}

		if jointsID != usize::MAX
		{
			let ja = &self.accessors[jointsID];
			let jbv = &self.bufferViews[ja.bufferView];
			let jb = &self.buffers[jbv.buffer];
			for i in 0..ja.count
			{
				let j = jbv.byteOffset + i * 4;
				joints.push(jb[j] as f32);
			}
		}

		if uvsID != usize::MAX
		{
			let uva = &self.accessors[uvsID];
			let uvbv = &self.bufferViews[uva.bufferView];
			let uvb = &self.buffers[uvbv.buffer];
			for i in 0..(uva.count * 2)
			{
				let uv = uvbv.byteOffset + i * 4;
				uvs.push(f32::from_le_bytes([uvb[uv], uvb[uv + 1], uvb[uv + 2], uvb[uv + 3]]));
			}
		}

		if materialID != usize::MAX
		{
			material = self.material(materialID).1;
		}
		
		(vertices, normals, elements, joints, uvs, material)
	}

	pub fn material(&self, id: usize) -> (String, u32)
	{
		let m = &self.materials[id];
		let t = &self.textures[m.texture];
		let s = &self.samplers[t.sampler];
		let i = &self.images[t.source];
		(m.name.clone(), Window::getTexture(
			i.uri.clone(),
			s.minFilter, s.magFilter
		))
	}
}