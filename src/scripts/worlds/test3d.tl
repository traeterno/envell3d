global AxisLength: number

global function Init()
	network.connect("127.0.0.1:26225")
	while not network.isReady() do end
	while not network.hasMessage("setup") do end
	network.setup(network.getMessage("setup"))

	world.spawn("checkpoint", "res/scripts/ents/test.lua", {
		path = "res/objects/checkpoint.gltf",
	})
	local c = 5
	for i = 1, c do
		world.spawn("hero"..tostring(i), "res/scripts/ents/hero.lua", {
			id = i - 1
		})
	end
	AxisLength = 20
end

global function Update()
	if window.keyJustPressed("Q") then window.close() end
	shaders.setVec3("mesh", "cam", 0, -1, 0)
end