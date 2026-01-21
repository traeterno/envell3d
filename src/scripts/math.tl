local function Clamp(x: number, min: number, max: number): number
	if x < min then return min end
	if x > max then return max end
	return x
end

local function Contains(px: number, py: number, x: number, y: number, w: number, h: number): boolean
	return (px == Clamp(px, x, x + w)) and (py == Clamp(py, y, y + h))
end

local function Intersects(hb1: Rectangle2D, hb2: Rectangle2D): boolean
	local il = math.max(hb1.x, hb2.x)
	local it = math.max(hb1.y, hb2.y)
	local ir = math.min(hb1.x + hb1.w, hb2.x + hb2.w)
	local ib = math.min(hb1.y + hb1.h, hb2.y + hb2.h)
	return (il < ir) and (it < ib)
end

local function Lerp(a: number, b: number, t: number): number
	return a + (b - a) * t
end

local function CubicInOut(a: number, b: number, t: number): number
	local x: number = 0
	if t < 0.5 then
		x = 4 * t * t * t
	else
		x = 1 - (-2 * t + 2)^3 / 2
	end
	return a + (b - a) * x
end

local function CubicIn(a: number, b: number, t: number): number
	return a + (b - a) * t^3
end

local function CubicOut(a: number, b: number, t: number): number
	return a + (b - a) * (1.0 - (1.0 - t)^3)
end

local function SineIn(a: number, b: number, t: number): number
	return a + (b - a) * (1 - math.cos((t * math.pi) / 2))
end

local function SineOut(a: number, b: number, t: number): number
	return a + (b - a) * math.sin((t * math.pi) / 2)
end

local function SineInOut(a: number, b: number, t: number): number
	return a + (b - a) * -(math.cos(math.pi * t) - 1) / 2
end

local function PolarToCartesian(r: number, a: number): (number, number)
	local angle: number = math.rad(a)
	return math.cos(angle) * r, math.sin(angle) * r
end

local function Round(x: number, digits: number): number
	return math.floor(x * 10^digits + 0.5) / 10^digits
end

return
{
	Clamp = Clamp,
	Contains = Contains,
	Intersects = Intersects,
	Lerp = Lerp,
	CubicInOut = CubicInOut,
	CubicIn = CubicIn,
	CubicOut = CubicOut,
	SineInOut = SineInOut,
	SineIn = SineIn,
	SineOut = SineOut,
	PolarToCartesian = PolarToCartesian,
	Round = Round
}