global Math = require("res.scripts.math")
global Global = require("res.scripts.global")

global Offset: number
global _: any
global Input: string

global Selected: boolean
global Hovered: boolean
global Visible: boolean

global WinW, WinH: number, number
global Scale, ScaleX, ScaleY: number, number, number

global Timer: number

global function Init()
	text.setString(">")
	Offset, _ = text.size()
	Offset = Offset + 1
	text.setString("")
	Selected = false
	Hovered = false
	Input = ""
	Visible = false
end

global function OnResized(sx: number, sy: number)
	WinW, WinH = window.size()
	ScaleX = sx
	ScaleY = sy
	Scale = (ScaleX + ScaleY) * 0.5
	text.setTransform({
		scale = { x = Scale, y = Scale },
		pos = {
			x = WinW * 0.125,
			y = WinH * vars.pos as number
		}
	})
end

global function Update()
	if not Visible then return end
	local mx, my = window.mousePos()
	local x, y, w, h = text.bounds()

	Hovered = Math.Contains(mx, my, x, y, w, h)

	if window.mouseJustPressed("Left") then
		Selected = Hovered
	end

	if Selected then
		Input = Global.str.input(Input, Global.str.symbols.Digits..".:")

		if window.keyJustPressed("Enter") then
			window.execute("ui_title", "Input(\""..Global.getID().."\",\""..Input.."\")")
		end
	end
end

global function Draw()
	if not Visible then return end
	if Selected then
		text.setString(">"..Input)
	elseif Hovered then
		if Input == "" then text.setString(">___")
		else text.setString(">"..Input) end
	else
		if Input == "" then text.setString("___")
		else text.setString(Input) end
	end
	local _, h = text.size()
	text.setTransform({
		origin = { x = (Hovered or Selected) and Offset or 0, y = h * 0.5 }
	})
	text.draw()
end

global function Activate(active: boolean)
	Visible = active
	if not Visible then
		Input = ""
	end
	Timer = 0
end