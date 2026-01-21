global record Rectangle2D
    x: number
    y: number
    w: number
    h: number
end

global record Point3D
    x: number
    y: number
    z: number
end

global record Point2D
    x: number
    y: number
end

global record TransformAngle
    yaw: number
    pitch: number | nil
    roll: number | nil
end

global record Transform2D
    pos: Point2D | nil
    scale: Point2D | nil
    origin: Point2D | nil
end

global record Transform3D
    pos: Point3D | nil
    scale: Point2D | number | nil
    origin: Point2D | nil
    angle: TransformAngle | number | nil
    movement: Point3D | nil
    fly: Point3D | nil
    direction: Point3D | nil
end

global record ThirdPersonSettings
    distance: number
end

global record CameraMode
    firstPerson: table
    thirdPerson: ThirdPersonSettings
end

global record NetworkMessage
    tickRate: number
    id: number
    port: number
end

global type Vars = table

global record window
    size: function(): (number, number)
    mousePos: function(): (number, number)
    setMousePos: function(number, number)
    setClipboard: function(string)
    input: function(): string
    keyJustPressed: function(string): boolean
    keyModPressed: function(string): boolean
    keyPressed: function(string): boolean
    clipboard: function(): string
    dt: function(): number
    showCursor: function(boolean)
    isMaximized: function(): boolean
    isFocused: function(): boolean
    execute: function(string, boolean | string)
    mouseJustPressed: function(string): boolean
    close: function()
    loadUI: function(string)
end

global record mesh 
    load: function(string, integer)
    setTransform: function(Transform3D)
    draw: function()
    getTransform: function({string}): Transform3D
end

global record skeleton 
    load: function(string, integer)
    update: function()
    setAnimation: function(string)
end

global record camera 
    setFOV: function(integer)
    setTransform: function(Transform3D)
    getTransform: function({string}): Transform3D
    addTransform: function(Transform3D)
    setMode: function(CameraMode)
    setDistance: function(number)
    setScale: function(number)
end

global record shaders
    setVec3: function(string, string, number, number, number)
    setBool: function(string, string, boolean)
end

global record network 
    id: function(): integer
    search: function()
    discovered: function(): string
    getState: function(integer): (number, number, number, number, number)
    setState: function(number, number, number, number, number)
    connect: function(string)
    isReady: function(): boolean
    hasMessage: function(string): boolean
    getMessage: function(string): NetworkMessage
    setup: function(NetworkMessage)
end

global record profiler
    get: function(string): number
end

global record text
    setString: function(string)
    draw: function()
    setTransform: function(Transform2D)
    getTransform: function({string}): Transform2D
    setColor: function(number, number, number, number)
    size: function(): number, number
    bounds: function(): number, number, number, number
end

global record world
    load: function(string)
    spawn: function(string, string, table)
end

global ScriptID: string
global vars: Vars