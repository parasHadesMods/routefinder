Import "Utils/FindRoute.lua"

-- Notation
-- o = offset, number of rng increments (since reset)
-- c = chamber, chamber number
-- r = room, the full room object
-- i = index, into an array
-- _s = array of (eg. rs = array of rooms, cs = array of chamber numbers, etc.)

function SetupFindIncrementally(run, door, requirements, cStart, cEnd, oStart)
    local state = {}
    -- validate
    if requirements.SelectUpgrade == nil then
        print("WARNING: SelectUpgrade not provided, will select first boon!")
    end

    for ci=cStart+1,cEnd do
        if requirements["C" .. ci] == nil then
            print("Missing requirements for C" .. ci)
        end
    end

    -- clear previous state
    state = {}
    state.Increments = 0
    state.Requirements = requirements
    state.cStart = cStart
    state.cLastPrediction = cEnd - 1 -- to get results for C7, our last prediction is from C6
    state.cEnd = cEnd
    state.rssReached = {} -- by depth
    for i=cStart,cEnd do
        state.rssReached[i] = {}
    end
    table.insert(state.rssReached[cStart], 
        {
            Run = run,
            Seed = NextSeeds[1],
            Door = door,
            oMinimum = oStart,
            oNext = oStart
        }
    )
    return state
end

local function nextRooms(state, rCurrent, ci)
    local results = {}
    local requirements = state.Requirements["C" .. (ci + 1)]

    NextSeeds[1] = rCurrent.Seed
    for _, reward in pairs(PredictRoomOptions(rCurrent.Run, rCurrent.Door, { Min = rCurrent.oNext, Max = rCurrent.oNext })) do
        if matches(requirements.Room, reward) then
            reward.RoomName = rCurrent.Door.Room.Name
            reward.Run = MoveToNextRoom(rCurrent.Run, reward, rCurrent.Door)
            reward.Seed = NextSeeds[1]
            reward.oMinimum = reward.EstimatedEndOfRoomOffset
            reward.oNext = reward.oMinimum
            if not requirements.SkipReward then
                PickUpReward(reward.Run, state.Requirements.SelectUpgrade, reward)
            end
            for _, door in pairs(ExitDoors(reward.Run, requirements, reward)) do
                local doorReward = DeepCopyTable(reward)
                doorReward.Door = door
                table.insert(results, doorReward)
            end
        end
    end
    return results
end

local function incrementChamber(state, ci)
    for i, room in ipairs(state.rssReached[ci]) do
        local rsNext = nextRooms(state, room, ci)
        for _, rNext in pairs(rsNext) do
            rNext.iPrevious = i
            table.insert(state.rssReached[ci+1], rNext)
        end
        room.oNext = room.oNext + 1
    end
end

local function stateIncrement(state)
    state.Increments = state.Increments + 1
    for ci=state.cStart,state.cLastPrediction do
        incrementChamber(state, ci)
    end
end

local function stateResults(state)
    local results = {}
    for _, room in pairs(state.rssReached[state.cEnd]) do
        local result = {}
        result["C" .. state.cEnd] = room
        local iPrevious = room.iPrevious
        for ci=state.cEnd-1,state.cStart,-1 do
            result["C" .. ci] = state.rssReached[ci][iPrevious]
            iPrevious = result["C" .. ci].iPrevious
        end
        table.insert(results, result)
    end
    return results
end

function FindIncrementally(states)
    local results = {}
    while #results == 0 do
        for _, state in pairs(states) do
            stateIncrement(state)
            for _, result in pairs(stateResults(state)) do
                table.insert(results, result)
            end
        end
    end
    return results
end