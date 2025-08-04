Import "Utils/DeepPrint.lua"

function ReadCheckpoint(filename)
    local infile = io.open(filename, "r")
    if infile ~= nil then
        infile:close()
        return LuabinsRead(filename)
    end
end

function SaveCheckpoint(filename, state)
    -- only present if LazyDeepCopy was imported by caller
    if LazyDeepCopyDeepUnstub then
        LazyDeepCopyDeepUnstub(state)
    end
    LuabinsWrite(filename, state)
end

function Checkpoint(filename, func)
    local result = ReadCheckpoint(filename)
    if result ~= nil then
        return result
    else
        result = func()
        SaveCheckpoint(filename, result)
        return result
    end
end