local function RemoveLastChar(str: string): string
	if #str == 0 then return str end
	local len = utf8.len(str, -1, -1)
	if len == nil then str = string.sub(str, 1, -3)
	else str = string.sub(str, 1, -2) end
	return str
end

local Symbols = {
	Latin = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz",
	Cyrillic = "АБВГДЕЁЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯабвгдеёжзийклмнопрстуфхцчшщъыьэюя",
	Digits = "0123456789",
	Special = " .,:;|/\\[]()!?-_+=^'\""
}

local function HandleInput(base: string, allowed: string): string
	local i = window.input()
	local f, _ = string.find(allowed, i, 1, true)
	if window.keyJustPressed("V") and window.keyModPressed("Control") then
		local clip = window.clipboard()
		for c = 1, #clip do
			local a, _ = string.find(allowed, string.sub(clip, c, c))
			if a ~= nil then base = base..string.sub(clip, c, c) end
		end
	elseif window.keyJustPressed("C") and window.keyModPressed("Control") then
		window.setClipboard(base)
	elseif window.keyJustPressed("X") and window.keyModPressed("Control") then
		window.setClipboard(base)
		base = ""
	elseif window.keyJustPressed("Backspace") then
		if window.keyModPressed("Control") then
			if string.sub(base, -1) == " " then
				base = RemoveLastChar(base)
			end
			while #base > 0 and string.sub(base, -1) ~= " " do
				base = RemoveLastChar(base)
			end
		else
			base = RemoveLastChar(base)
		end
	elseif f ~= nil then base = base..i end

	return base
end

local function GetID(): string
	local f = string.find(ScriptID, "_")
	if f ~= nil then return string.sub(ScriptID, f + 1) end
	return ""
end

local function TrimString(s: string): string
	if #s == 0 then return "" end
	while string.sub(s, 1, 1) == " " do
		s = string.sub(s, 2)
	end
	while string.sub(s, -1, -1) == "" do
		s = string.sub(s, 1, -2)
	end
	return s
end

local function Split(str: string, sep: string, plain?: string): {string}
	local list: {string} = {}
	local quoted = false
	local s = ""
	for i = 1, #str do
		local c = string.sub(str, i, i)
		if c == sep then
			if quoted then s = s..c
			elseif #s ~= 0 then
				table.insert(list, s)
				s = ""
			end
		elseif c == '"' and plain == nil then
			quoted = not quoted
			if #s ~= 0 then
				table.insert(list, s)
				s = ""
			end
		else s = s..c end
	end
	if #s ~= 0 then table.insert(list, s) end
	return list
end

local function JoinString(parts: {string}, sep: string): string
	local out = #parts >= 1 and parts[1] or ""
	for i = 2, #parts do
		out = out..sep..parts[i]
	end
	return out
end

local function HSVtoRGB(h: number, s: number, v: number): number, number, number
	local r, g, b: number, number, number
	if s == 0 then
		r = math.floor(v * 255)
		g = math.floor(v * 255)
		b = math.floor(v * 255)
	else
		local c = s * v
		local x = c * (1 - math.abs((h / 60) % 2 - 1))
		local m = v - c
		if h >= 0 and h < 60 then r = c; g = x; b = 0
		elseif h >= 60 and h < 120 then r = x; g = c; b = 0
		elseif h >= 120 and h < 180 then r = 0; g = c; b = x
		elseif h >= 180 and h < 240 then r = 0; g = x; b = c
		elseif h >= 240 and h < 300 then r = x; g = 0; b = c
		elseif h >= 300 and h < 360 then r = c; g = 0; b = x
		end
		r = math.floor((r + m) * 255)
		g = math.floor((g + m) * 255)
		b = math.floor((b + m) * 255)
	end
	return r, g, b
end

local function UnpackTable(t: table, base?: string): table
	local out: table = {}
	-- local prefix = base == nil and "" or base.."."
	for k, p in pairs(t) do
		if base == nil then
			if type(p) == "table" then
				for k1, p1 in pairs(UnpackTable(p as table, k as string)) do
					table.insert(out as table, { k = k1, p = p1 })
				end
			else table.insert(out as table, { k = k, p = p }) end
		else
			if type(p) == "table" then
				for k1, p1 in pairs(UnpackTable(p as table, base.."."..k)) do
					table.insert(out as table, { k = k1, p = p1 })
				end
			else table.insert(out as table, { k = base.."."..k, p = p }) end
		end
		-- if type(p) == "table" then
		-- 	for k1, p1 in pairs(UnpackTable(p, prefix..k)) do
		-- 		table.insert(out, { k = k1, p = p1 })
		-- 	end
		-- else table.insert(out, { k = prefix..k, p = p }) end
	end
	return out
end

local function DebugTable(t: table, base: string)
	for _, x in ipairs(UnpackTable(t, base)) do
		print(x.k, x.p)
	end
end

local function ResetCursor(): number, number
	local sx, sy = window.size()
	local cx = sx * 0.5
	local cy = sy * 0.5

	local mx, my = window.mousePos()
	window.setMousePos(cx, cy)
	return mx - cx, my - cy
end

return {
	str = {
		removeLastChar = RemoveLastChar,
		symbols = Symbols,
		input = HandleInput,
		trim = TrimString,
		split = Split,
		join = JoinString
	},
	getID = GetID,
	resetCursor = ResetCursor,
	color = {
		hsv2rgb = HSVtoRGB
	},
	table = {
		debug = DebugTable,
		unpack = UnpackTable
	}
}