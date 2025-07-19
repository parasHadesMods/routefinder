local state = {}

-- Notation
-- o = offset, number of rng increments (since reset)
-- c = chamber, chamber number
-- r = room, the full room object
-- _s = array of (eg. rs = array of rooms, cs = array of chamber numbers, etc.)

function Setup(run, door, requirements, cStart, cEnd, oStart)
    -- clear previous state
    state = {}
    state.Requirements = requirements
    state.cStart = cStart
    state.cEnd = cEnd
    state.rssReached = {} -- by depth
    for i=cStart,cEnd+1 do
        state.rssReached[i] = {}
    end
    table.insert(state.rssReached[cStart], 
        {
            Run = run,
            Door = door,
            Display = "",
            oMinimum = oStart,
            oNext = oStart
        }
    )
end

function NextRooms(rCurrent)
    return {
        {
            Display = rCurrent.Display .. " " .. (rCurrent.oNext - rCurrent.oMinimum),
            oMinimum = 7,
            oNext = 7
        }
    }
end

function IncrementChamber(ci)
    for _, room in pairs(state.rssReached[ci]) do
        local rsNext = NextRooms(room)
        for _, rNext in pairs(rsNext) do
            table.insert(state.rssReached[ci+1], rNext)
        end
        room.oNext = room.oNext + 1
    end
end

function Increment()
    for ci=state.cStart,state.cEnd do
        IncrementChamber(ci)
    end
end