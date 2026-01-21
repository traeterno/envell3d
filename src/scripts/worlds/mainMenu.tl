global Timer: number

global function Init()
	world.spawn("cube", "res/scripts/objects/cheatCube.lua", {})
	-- world.spawn("cam", "res/scripts/ents/noclip.lua", {})
	camera.setFOV(70)
	camera.setDistance(0.5)
	camera.setTransform({
		pos = {},
		angle = {
			pitch = 30
		}
	})
	camera.setScale(2)
	local dir = camera.getTransform({"direction"}).direction as Point3D
	shaders.setVec3("mesh", "lightDir", dir.x, dir.y, dir.z)
	shaders.setBool("mesh", "useLighting", true)
	Timer = 0
end

global function Update()
	if window.keyJustPressed("F1") then
		window.loadUI("res/ui/mainMenu.json")
	end
	if window.keyJustPressed("Q") and window.keyModPressed("Control") then
		window.close()
	end
end