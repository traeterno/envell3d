global Math = require("res.scripts.math")

global Update: function()
global Timer: number
global State: string
global StartAngle: TransformAngle

local record Data

end

global function MainUpdate()
	Timer = Timer + window.dt()
	local a = Timer * math.pi * 0.25
	local amplitude = 10
	mesh.setTransform({
		angle = {
			yaw = amplitude * 2 * math.cos(a * 0.5),
			pitch = amplitude * math.sin(a),
			roll = amplitude * math.sin(a * 0.5)
		}
	})

	if Timer % 5 < 0.05 then
		skeleton.setAnimation(State)
		if State == "solving" then State = "dissolving"
		else State = "solving" end
	end
end

global function Draw()
	skeleton.update()
	mesh.draw()
end

global function Entrance()
	local duration = 4
	Timer = Timer + window.dt()
	local t = Math.CubicInOut(0, 1, Math.Clamp(Timer, 0, duration) / duration)
	mesh.setTransform({
		scale = t,
		angle = {
			yaw = 20 - 360 * t,
			pitch = -360 * t
		}
	})
	if Timer > duration then
		Timer = 0
		Update = MainUpdate
		-- Update = Testing
	end
end

global function Init(_: Data)
	mesh.load("res/objects/cheatCube.gltf", 0)
	skeleton.load("res/objects/cheatCube.gltf", 0)

	Update = Entrance
	Timer = 0.0
	State = "solving"
end

global function Translate(f: function())
	StartAngle = mesh.getTransform({"angle"}).angle as TransformAngle
	Timer = 0
	Update = f
end

global function Exit()
	local duration = 4
	Timer = Timer + window.dt()
	local t = Math.CubicInOut(1, 0, Math.Clamp(Timer, 0, duration) / duration)
	mesh.setTransform({
		scale = t,
		angle = {
			yaw = StartAngle.yaw - 360 * t,
			pitch = StartAngle.pitch - 360 * t
		}
	})
end

global function Work()
	Timer = Timer + window.dt()
	local speed = 360 * Math.CubicInOut(0, 1, Math.Clamp(Timer, 0, 1))
	local x = mesh.getTransform({"angle"}).angle as TransformAngle
	mesh.setTransform({
		angle = {
			yaw = x.yaw + speed * window.dt(),
			pitch = Math.Lerp(x.pitch, 30, window.dt())
		}
	})
end

global function StopWorking()
	Timer = Timer + window.dt()
	-- local speed = 360 * Math.CubicInOut(1, 0, Math.Clamp(Timer, 0, 1))
	-- local x = mesh.getTransform({"angle"}).angle
	mesh.setTransform({})
end
-- 127.0.0.1:26225

global function Testing()
	mesh.setTransform({
		pos = {},
		angle = {},
		scale = 1
	})
end