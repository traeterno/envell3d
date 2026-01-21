global Math = require("res.scripts.math")
global Global = require("res.scripts.global")

global record StatusRecord
    Text: string
    Time: number
end

global record SearchingRecord
    Time: number
    Attempts: integer
    Servers: {any}
end

global Timer: number
global Connecting: boolean
global Update: function()
global Status: StatusRecord
global Searching: SearchingRecord

global WinW, WinH: number, number
global Scale, ScaleX, ScaleY: number, number, number

global function OnResized(sx: number, sy: number)
	WinW, WinH = window.size()
	ScaleX = sx
	ScaleY = sy
	text.setTransform({
		scale = {
			x = (ScaleX + ScaleY) * 0.5,
			y = (ScaleX + ScaleY) * 0.5
		},
		pos = {
			x = WinW * 0.5,
			y = WinH * 0.125
		}
	})
end

global function Draw()
	text.draw()
end

global function MainUpdate()
	if Status.Time > 0 then
		Status.Time = Status.Time - window.dt()
		text.setString(Status.Text)
	else text.setString("Герои Энвелла") end
	local w, h = text.size()
	text.setTransform({
		origin = { x = w * 0.5, y = h * 0.5 }
	})

	if Searching.Attempts >= 0 then
		Status.Time = 2
		Status.Text = "Ищем сервер"..string.rep(".", (5 - Searching.Attempts))
		Searching.Time = Searching.Time + window.dt()
		if Searching.Time >= 2 then
			Searching.Attempts = Searching.Attempts - 1
			Searching.Time = 0
		end
		local ip = network.discovered()
		if ip ~= "" then
			print("New IP: "..ip)
			-- TODO list of servers
			-- TODO remake of 'Searching...' title
		end
	end
end

global function ToggleMenu(menu, active)
	local status: string
	if active then status = "true" else status = "false" end

	for _, btn in ipairs(Global.str.split(menu as string, "|")) do
		window.execute("ui_"..btn, "Activate("..status..")")
	end
end

global function Entrance()
	local duration = 4
	Timer = Timer + window.dt()
	if Timer > duration then
		Update = MainUpdate
		ToggleMenu(vars.main, true)
	end
	text.setTransform({
		pos = {
			x = WinW * 0.5,
			y = Math.CubicInOut(
				-WinH,
				WinH * 0.125,
				Math.Clamp(Timer, 0, duration) / duration
			)
		}
	})
end

global function Exit()
	local duration = 4
	Timer = Timer + window.dt()
	if Timer > duration then
		window.close()
	end
	text.setTransform({
		pos = {
			x = WinW * 0.5,
			y = Math.CubicInOut(
				WinH * 0.125,
				-WinH,
				Math.Clamp(Timer, 0, duration) / duration
			)
		}
	})
end

global function Search()
	Searching.Attempts = 4
	Searching.Time = 0
	network.search()
end

global function Btn(id)
	if id == "search" then
		ToggleMenu(vars.main, false)
		ToggleMenu(vars.search, true)
	elseif id == "exit" then
		window.execute("ent_cube", "Translate(Exit)")
		ToggleMenu(vars.main, false)
		Timer = 0
		Update = Exit
	elseif id == "toMain" then
		ToggleMenu(vars.search, false)
		ToggleMenu(vars.main, true)
	elseif id == "repeatSearch" then
		Search()
	else print(id) end
end

global function Input(id: string, txt: string)
	if id == "enterIP" then
		network.connect(txt)
		Status.Time = 10
		Status.Text = "Подключаемся к "..txt.."..."
		Connecting = true
		window.execute("ent_cube", "Translate(Work)")
	end
end

global function Init()
	world.load("mainMenu")
	local w, h = text.size()
	text.setTransform({
		origin = { x = w * 0.5, y = h * 0.5 }
	})
	Timer = 0
	Update = Entrance
	Status = { Text = "", Time = 0 }
	Searching = { Time = 0, Attempts = -1, Servers = {} }
end