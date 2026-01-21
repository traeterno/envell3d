local record Data
	path: string
	x: number
	y: number
	z: number
	scale: number
	anim: string
	useLighting: boolean
end

global Animated: boolean
global UseLighting: boolean

global function Init(data: Data)
	mesh.load(data.path, 0)
	mesh.setTransform({
		pos = {
			x = data.x or 0,
			y = data.y or 0,
			z = data.z or 0
		},
		scale = data.scale or 1.0
	})
	Animated = data.anim ~= nil
	if Animated then
		skeleton.load(data.path, 0)
		skeleton.setAnimation(data.anim)
	end
	UseLighting = data.useLighting
	if UseLighting == nil then UseLighting = true end
end

global function Update() end

global function Draw()
	if Animated then skeleton.update() end
	shaders.setBool("mesh", "useLighting", UseLighting)
	mesh.draw()
end