Import "Utils/FindRoute.lua"
local state = {}

-- Notation
-- o = offset, number of rng increments (since reset)
-- c = chamber, chamber number
-- r = room, the full room object
-- _s = array of (eg. rs = array of rooms, cs = array of chamber numbers, etc.)

function Setup(run, door, requirements, cStart, cEnd, oStart)
    -- clear previous state
    state = {}
    state.Increments = 0
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
            Seed = NextSeeds[1],
            Door = door,
            Display = "",
            oMinimum = oStart,
            oNext = oStart
        }
    )
end

function NextRooms(rCurrent, ci)
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
                PickUpReward(reward.Run, requirements.Boon, reward)
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

function IncrementChamber(ci)
    for _, room in pairs(state.rssReached[ci]) do
        local rsNext = NextRooms(room, ci)
        for _, rNext in pairs(rsNext) do
            table.insert(state.rssReached[ci+1], rNext)
        end
        room.oNext = room.oNext + 1
    end
end

function Increment()
    state.Increments = state.Increments + 1
    for ci=state.cStart,state.cEnd do
        IncrementChamber(ci)
    end
end

function MonitorProgress()
    if state.Increments % 10 == 0 then
        local progress = state.Increments .. ": " .. #state.rssReached[state.cStart]
        for ci=state.cStart+1,state.cEnd do
            progress = progress .. " " .. #state.rssReached[ci]
        end
        print(progress)
    end
end

function Results()
    return state.rssReached[state.cEnd]
end