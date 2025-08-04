Import "Utils/Checkpoint.lua"
Import "Utils/FindIncrementally.lua"

function NewRequirements(cStart, cEnd)
    local r = {}
    for ci=cStart,cEnd do
        local cid = "C" .. ci
        r[cid] = { Room = {}, Exit = {} }
    end
    return r
end

local Upgrades = { "AresWeaponTrait", "AthenaSecondaryTrait", "TriggerCurseTrait", "AresLongCurseTrait" }
function SelectUpgrade(options)
  for _, option in ipairs(options) do
      for _, requiredItemName in ipairs(Upgrades) do
        if option.ItemName == requiredItemName then
            return option
        end
      end
  end
  return options[1]
end

function Display(route)
    local display = {}
    for ci=1,50 do
        local thisRoom = route["C" .. ci]
        local nextRoom = route["C" .. (ci + 1)]

        if thisRoom ~= nil and nextRoom ~= nil then
        local current = {}
        if thisRoom.UpgradeOptions ~= nil then
            local selected = SelectUpgrade(thisRoom.UpgradeOptions)
            current.Take = selected.ItemName .. " " .. selected.Rarity
        end
        current.Cast = nextRoom.Uses - thisRoom.oMinimum
        current.Door = thisRoom.Door.Room.ForceLootName or thisRoom.Door.Room.ChosenRewardType
        current.Seed = thisRoom.Seed
        display[ci] = current
        end
    end
    deep_print(display)
end

function FindIncrementallyWithForcedOffset(rResumeFrom, requirements, cStart, cEnd, oWiggleRoom, forcedOffset)
    local results = {}
    NextSeeds[1] = rResumeFrom.Seed
    local nextRoomRequirements = requirements["C".. (cStart + 1)]
    for _, reward in pairs(PredictRoomOptions(rResumeFrom.State, rResumeFrom.Door, { Min = forcedOffset, Max = forcedOffset})) do
        -- we can't check requirements, but we still have the option of which exit door to choose
        reward.RoomName = rResumeFrom.Door.Room.Name
        reward.State = moveToNextRoom(rResumeFrom.State, reward, rResumeFrom.Door)
        reward.Seed = NextSeeds[1]
        reward.oMinimum = nextRoomRequirements.ForceMinimumOffset or reward.EstimatedEndOfRoomOffset
        reward.oNext = reward.oMinimum + (nextRoomRequirements.ExtraWiggleRoom or 0) + oWiggleRoom
        if not nextRoomRequirements.SkipReward then
            PickUpReward(reward.State.CurrentRun, requirements.SelectUpgrade, reward)
        end
        for _, door in pairs(ExitDoors(reward.State.CurrentRun, nextRoomRequirements, reward)) do
            local doorReward = DeepCopyTable(reward)
            doorReward.Door = door
            table.insert(results, doorReward)
        end
        if #results == 0 then
            -- try again but remove the exit door requirements in case neither is valid because we got here by mistake
            for _, door in pairs(ExitDoors(reward.State.CurrentRun, { Exit = {} }, reward)) do
                local doorReward = DeepCopyTable(reward)
                doorReward.Door = door
                table.insert(results, doorReward)
            end
        end
    end
    local states = {}
    for _, possibleStart in pairs(results) do
        table.insert(states, ResumeFindIncrementally(possibleStart, requirements, cStart + 1, cEnd, oWiggleRoom))
    end
    return FindIncrementally(states)
end

if FirstChamberOffRoute >= 19 then
    local checkpoint = ReadCheckpoint("checkpoints/finalRoute.bin")
    local lastChamberOnRoute = checkpoint["C" .. (FirstChamberOffRoute - 1)]
    local firstChamberOffRoute = checkpoint["C" .. FirstChamberOffRoute]

    if ActualOffset == nil then
        local range = { Min = firstChamberOffRoute.Uses + (OffsetOffBy or 1), Max = firstChamberOffRoute.Uses + (OffsetOffBy or 1) }
        NextSeeds[1] = lastChamberOnRoute.Seed
        for _, reward in pairs(PredictRoomOptions(lastChamberOnRoute.State, lastChamberOnRoute.Door, range)) do
            clean_reward(reward)
            reward.EstimatedEndOfRoomOffset = nil
            reward.UpgradeOptionsReroll = nil
            deep_print(reward)
        end
        return
    end
    ActualOffset = ActualOffset + firstChamberOffRoute.Uses

    -- Third section. All we care about is avoiding Dio and getting 2-sack.
    local requirements = NewRequirements(18, 48)
    for ci=18,46 do
    requirements["C"..ci].Exit.Reward = Not("DionysusUpgrade")
    end
    requirements.C24.Exit.Reward = nil -- C24 = Lernie, C25 = Stairs, C26 = elysium intro
    requirements.C25.ForceMinimumOffset = 12 -- well reset in stairs
    requirements.C25.Exit.Reward = nil
    requirements.C36.Exit.Reward = nil -- C36 = Heroes, C37 = Stairs, C38 = styx intro, C39 = styx hub
    requirements.C37.ForceMinimumOffset = 12 -- well reset in stairs
    requirements.C37.Exit.Reward = nil
    requirements.C38.Exit.Reward = nil
    requirements.C39.Exit.StyxMiniBoss = true
    -- need to force short tunnel / miniboss in 43 (not tony)
    requirements.C42.Exit.RoomName = MatchesOne({ "D_MiniBoss01", "D_MiniBoss04"})
    -- requirements.C43.Room.RoomName = MatchesOne({ "D_MiniBoss04", "D_MiniBoss01" })
    requirements.C47.Exit.RoomName = "D_Reprieve01" -- sack

    local reroute = FindIncrementallyWithForcedOffset(lastChamberOnRoute, requirements, FirstChamberOffRoute-1, 26, 1, ActualOffset)
    Display(reroute[1])
end