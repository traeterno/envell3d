global Math = require("res.scripts.math")
global Global = require("res.scripts.global")

local record Data

end

global DT: number
global LockCursor: boolean
global Sensitivity: number
global Speed: number

global function Init(_: Data)
	camera.setFOV(70)
	camera.setDistance(0)

	LockCursor = true
	Sensitivity = 0.2
	Speed = 0.25

	if window.isFocused() then
		window.showCursor(not LockCursor)
		Global.resetCursor()
	end
end

global function Update()
	DT = window.dt()
	if window.keyJustPressed("Tab") then
		LockCursor = not LockCursor
		window.showCursor(not LockCursor)
		if LockCursor then Global.resetCursor() end
	end

	if not (LockCursor and window.isFocused()) then return end

	local mx, my = Global.resetCursor()
	if window.isMaximized() then my = my + 0.5 end

	local dx = 0
	local dy = 0
	local dz = 0
	local factor = 1.0

	if window.keyPressed("A") then dx = dx - 1 end
	if window.keyPressed("D") then dx = dx + 1 end
	if window.keyPressed("W") then dz = dz + 1 end
	if window.keyPressed("S") then dz = dz - 1 end
	if window.keyPressed("Space") then dy = dy + 1 end
	if window.keyPressed("LCtrl") then dy = dy - 1 end
	if window.keyPressed("LShift") then factor = 2 end
	if window.keyPressed("LAlt") then factor = 0.5 end

	camera.addTransform({
		angle = {
			yaw = -mx * Sensitivity,
			pitch = my * Sensitivity
		},
		fly = {
			x = dx * Speed * factor * DT,
			y = dy * Speed * factor * DT,
			z = dz * Speed * factor * DT
		}
	})

	local dir = camera.getTransform({"direction"}).direction as Point3D
	shaders.setVec3("mesh", "lightDir", dir.x, dir.y, dir.z)
end