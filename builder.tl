local record Entry
	path: string
	isFile: boolean
end

local function split(str: string, sep: string, plain?: boolean): {string}
	local list = {}
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

local function listDir(path: string): {Entry}
	local f = io.popen("ls "..path, "r")
	local s = f:read("*all"):sub(1, -2)
	local out: {Entry} = {}
	for _, p in ipairs(split(s, "\n")) do
		local n = p:find(".", 0, true)
		table.insert(out, {
			path = p,
			isFile = (n ~= nil)
		})
	end
	return out
end

local function recursiveDirs(start: string): {string}
	local out: {string} = {}
	for _, entry in ipairs(listDir(start)) do
		if not entry.isFile then
			local path = start.."/"..entry.path
			out[#out + 1] = path
			local p = recursiveDirs(path)
			for _, e in ipairs(p) do
				out[#out + 1] = e
			end
		end
	end
	return out
end

local function recursiveFiles(start: string, ext: string): {string}
	local out: {string} = {}
	for _, entry in ipairs(listDir(start)) do
		local p = start.."/"..entry.path
		if entry.isFile then
			local n = entry.path:find("."..ext, 0, true)
			if n ~= nil then out[#out + 1] = p end
		else
			for _, e in ipairs(recursiveFiles(p, ext)) do
				out[#out + 1] = e
			end
		end
	end
	return out
end

local function clean()
	print("Confirm full cleaning.")
	local inp = io.read()
	if inp ~= "y" and inp ~= "yes" and inp ~= "Y" and inp ~= "YES" then
		print("Declined full cleaning.")
		os.exit(0)
	end
	for _, p in ipairs(recursiveFiles("./res/scripts", "lua")) do
		print("Deleting file \""..p.."\"...")
		os.execute("rm \""..p.."\"")
	end
	for _, p in ipairs(recursiveDirs("./res/scripts")) do
		print("Deleting directory \""..p.."\"...")
		os.execute("rm -rf \""..p.."\"")
	end
end

local function restruct()
	print("Clean repeating structure from source directory.")
	for _, d in ipairs(recursiveDirs("./res/scripts")) do
		os.execute("rm  -rf \""..d.."\"")
	end
	for _, start in ipairs(recursiveDirs("./src/scripts")) do
		local target = "./res/scripts/"..start:sub(15)
		print(start.." -> "..target)
		os.execute("mkdir \""..target.."\"")
	end
end

local function compile()
	print("Compiling all sources...")
	for _, f in ipairs(recursiveFiles("./src/scripts", "tl")) do
		local target = "./res/scripts"..f:sub(14, -4)..".lua"
		print(f)
		os.execute("tl gen \""..f.."\" -o \""..target.."\"")
	end
end

local mode = arg[1] or "all"

if mode == "clean" then clean()
elseif mode == "restruct" then restruct()
elseif mode == "compile" then compile()
elseif mode == "all" then
	print("Cleaning output directory and compiling all sources.")
	clean()
	restruct()
	compile()
else
	local f = mode
	print("Building source \""..f.."\".")
	local target = "./res/scripts"..f:sub(14, -4)..".lua"
	os.execute("tl gen \""..f.."\" -o \""..target.."\"")
end