local record Data
	id: integer
end

global ID: integer
global Control: boolean
global Lock: boolean
global DT: number
global Speed: number
global Sensitivity: number

global function ResetCursor(): (number, number)
	local sx, sy = window.size()
	local cx = sx * 0.5
	local cy = sy * 0.5

	local mx, my = window.mousePos()
	window.setMousePos(cx, cy)
	return mx - cx, my - cy
end

global function Init(data: Data)
	mesh.load("res/objects/hero.gltf", 0)
	skeleton.load("res/objects/hero.gltf", 0)
	ID = data.id
	Control = ID == network.id()

	if Control then
		camera.setFOV(70)
		camera.setTransform({
			pos = {
				x = 0,
				y = 0.25 + 1.5,
				z = 0
			}
		})
		camera.setMode({ thirdPerson = { distance = 3 } })
		Sensitivity = 0.2
		Speed = 2
		Lock = false
	end
end

global function Update()
	if not Control then
		local x, y, z, yaw, _ = network.getState(ID)
		mesh.setTransform({
			pos = { x = x, y = y, z = z },
			angle = { yaw = -90 - yaw }
		})
		return
	end

	DT = window.dt()

	if window.keyJustPressed("Tab") then
		Lock = not Lock
		if Lock then ResetCursor() end
		window.showCursor(not Lock)
	end

	if not Lock then return end

	local mx, my = ResetCursor()
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
	if window.keyPressed("LShift") then factor = factor * 2 end
	if window.keyPressed("LAlt") then factor = factor * 0.5 end

	camera.addTransform({
		angle = {
			yaw = mx * Sensitivity,
			pitch = -my * Sensitivity
		},
		movement = {
			x = dx * factor * DT,
			y = dy * factor * DT,
			z = dz * factor * DT
		}
	})

	local ts = camera.getTransform({ "pos", "angle" })
	local pos = ts.pos as Point3D
	local angle = ts.angle as TransformAngle
	local pitch = angle.pitch as number

	mesh.setTransform({
		pos = {
			x = pos.x,
			y = pos.y - 1.5,
			z = pos.z
		},
		angle = {
			yaw = -90 - angle.yaw
		}
	})

	network.setState(
		pos.x,
		pos.y - 1.5,
		pos.z,
		angle.yaw,
		pitch
	)
end

global function Draw()
	skeleton.update()
	mesh.draw()
end