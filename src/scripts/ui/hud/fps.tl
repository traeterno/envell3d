global Math = require("res.scripts.math")

global Period: number
global Timer: number
global Profiler: string

global function Init()
	Period = 0.1
	Timer = Period
	Profiler = ""
end

global function Update()
	Timer = Timer + window.dt()
	if Timer > Period then
		Timer = 0
		local wu = Math.Round(profiler.get("worldUpdate"), 5)
		local wd = Math.Round(profiler.get("worldDraw"), 5)
		local uiu = Math.Round(profiler.get("uiUpdate"), 5)
		local uid = Math.Round(profiler.get("uiDraw"), 5)
		local winu = Math.Round(profiler.get("winUpdate"), 5)
		local r = Math.Round(profiler.get("render"), 5)
		local s = Math.Round(profiler.get("swap"), 5)
		local c = Math.Round(profiler.get("clear"), 5)
		local estimated = 1.0 / (wu + wd + uiu + uid + winu + s + c)
		Profiler = "FPS: "..tostring(math.floor(1.0 / window.dt()))
			.."\nWU "..tostring(wu)
			.."\nWD "..tostring(wd)
			.."\nUIU "..tostring(uiu)
			.."\nUID "..tostring(uid)
			.."\nWinU "..tostring(winu)
			.."\nR "..tostring(r)
			.."\nS "..tostring(s)
			.."\nC "..tostring(c)
			.."\nE "..tostring(math.floor(estimated))
	end
end

global function Draw()
	text.setString(Profiler)
	text.draw()
end
