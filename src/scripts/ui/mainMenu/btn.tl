global Math = require("res.scripts.math")
global Global = require("res.scripts.global")

global Visible: boolean
global WasVisible: boolean
global Timer, DT: number, number
global WinW, WinH: number, number
global Scale, ScaleX, ScaleY: number, number, number

global function Init()
	local w, h = text.size()
	text.setTransform({
		origin = { x = w * 0.5, y = h * 0.5 }
	})

	Visible = false
	WasVisible = false

	Timer = 0
	text.setColor(255, 255, 255, 0)
end

global function Draw()
	text.draw()
end

global function Animate()
	local t = Math.CubicInOut(0, 1, Math.Clamp(Timer, 0, 0.5) * 2)
	if Timer > 0.5 then
		WasVisible = Visible
		if Visible and vars.onVisible ~= nil then
			window.execute("ui_title", vars.onVisible as boolean)
		end
		return
	end

	if not WasVisible and Visible then
		text.setColor(255, 255, 255, 255 * t)
	elseif WasVisible and not Visible then
		text.setColor(255, 255, 255, 255 * (1 - t))
	end
end

global function Activate(active: boolean)
	Visible = active
	Timer = 0
end

global function OnResized(sx: number, sy: number)
	WinW, WinH = window.size()
	ScaleX = sx
	ScaleY = sy
	Scale = (ScaleX + ScaleY) * 0.5
	text.setTransform({
		scale = {
			x = Scale,
			y = Scale
		},
		pos = {
			x = WinW * 0.2,
			y = WinH * vars.pos as number
		}
	})
end

global function Update()
	if not window.isFocused() then return end
	DT = window.dt()
	Timer = Timer + DT

	if WasVisible ~= Visible then
		Animate()
		return
	end

	local x, y, w, h = text.bounds()
	local mx, my = window.mousePos()

	local scale = text.getTransform({"scale"}).scale as Point2D
	if Math.Contains(mx, my, x, y, w, h) and Visible and WasVisible then
		text.setTransform({
			scale = {
				x = Math.Lerp(scale.x, Scale * 1.1, DT * 10),
				y = Math.Lerp(scale.y, Scale * 1.1, DT * 10)
			}
		})

		if window.mouseJustPressed("Left") then
			window.execute("ui_title", "Btn(\""..Global.getID().."\")")
		end
	else
		text.setTransform({
			scale = {
				x = Math.Lerp(scale.x, Scale, DT * 10),
				y = Math.Lerp(scale.y, Scale, DT * 10)
			}
		})
	end
end