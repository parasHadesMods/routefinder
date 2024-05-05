local LazyDeepCopy = {}

LazyDeepCopy.Wrap = function(baseFunc)
    return function(t, ...)
        LazyDeepCopy.Unstub(t)
        return baseFunc(t, ...)
    end
end

local StubMetaTable = {
    __index = function(t, k)
        LazyDeepCopy.Unstub(t)
        return t[k]
    end,
    __newindex = function(t, k, v)
        LazyDeepCopy.Unstub(t)
        t[k] = v
    end,
    __pairs = LazyDeepCopy.Wrap(pairs),
    __ipairs = LazyDeepCopy.Wrap(ipairs),
    __next = LazyDeepCopy.Wrap(next),
    __len = function(t)
        LazyDeepCopy.Unstub(t)
        return #t
    end
}

table.concat = LazyDeepCopy.Wrap(table.concat)
table.insert = LazyDeepCopy.Wrap(table.insert)
-- todo: table.move ? not used by sgg
table.remove = LazyDeepCopy.Wrap(table.remove)
table.sort = LazyDeepCopy.Wrap(table.sort)
table.unpack = LazyDeepCopy.Wrap(table.unpack)

LazyDeepCopy.Unstub = function(t)
    if getmetatable(t) == StubMetaTable then
        setmetatable(t, nil)
        local target = t.__target
        t.__target = nil
        for k,v in pairs(target) do
            if type(v) == "table" then
                t[k] = LazyDeepCopyTable(v)
            else
                t[k] = v
            end
        end
    end
end

function LazyDeepCopyTable(t)
    local stub = { __target = t.__target or t }
    setmetatable(stub, StubMetaTable)
    return stub
end