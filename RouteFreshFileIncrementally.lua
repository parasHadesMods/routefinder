Import "Utils/FindIncrementally.lua"
Import "Utils/LazyDeepCopyTable.lua"
DeepCopyTable = LazyDeepCopyTable

local C2Door = CreateC2Door({
    SecondRoomName = "RoomSimple01", -- Athena Room
    SecondRoomReward = "Athena",
    SecondRoomRewardStore = "RunProgress"
})
C2Door.Room.Encounter = {}

local AthenaDashTrait = GetProcessedTraitData({ Unit = CurrentRun.Hero, TraitName = "AthenaRushTrait", Rarity = "Common" })
table.insert(CurrentRun.Hero.Traits, AthenaDashTrait)
CurrentRun.LootTypeHistory["AthenaUpgrade"] = 1

if type(AthenaSeed) ~= "number" then
  print("Invalid seed, make sure to pass --lua-var AthenaSeed=<number>")
end

if type(AthenaOffset) ~= "number" then
  print("Invalid offset, make sure to pass --lua-var AthenaOffset=<number>")
end

NextSeeds[1] = AthenaSeed

function NewRequirements(cStart, cEnd)
  local r = {}
  for ci=cStart,cEnd do
    local cid = "C" .. ci
    r[cid] = { Room = {}, Exit = {} }
  end
  return r
end

local Upgrades = { "AresWeaponTrait", "AthenaSecondaryTrait", "TriggerCurseTrait" }
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

local requireAresFirst = NewRequirements(3, 7)
requireAresFirst.SelectUpgrade = SelectUpgrade
requireAresFirst.C3.Exit.Reward = "AresUpgrade"
requireAresFirst.C4.Room.UpgradeOptions = OneMatches({
  ItemName = "AresWeaponTrait",
  Rarity = MatchesOne({ "Epic" })
})
requireAresFirst.C5.Exit.Reward = "AthenaUpgrade"
requireAresFirst.C6.Room.UpgradeOptions = OneMatches({
  ItemName = "AthenaSecondaryTrait"
})
requireAresFirst.C6.Exit.RoomName = "A_MiniBoss01"
requireAresFirst.C7.Room.UpgradeOptions = OneMatches({
  ItemName = "TriggerCurseTrait",
  Rarity = "Legendary"
})

local requireAthenaFirst = NewRequirements(3, 7)
requireAthenaFirst.SelectUpgrade = SelectUpgrade
requireAthenaFirst.C3.Exit.Reward = "AthenaUpgrade"
requireAthenaFirst.C4.Room.UpgradeOptions = OneMatches({
  ItemName = "AthenaSecondaryTrait"
})
requireAthenaFirst.C5.Exit.Reward = "AresUpgrade"
requireAthenaFirst.C6.Room.UpgradeOptions = OneMatches({
  ItemName = "AresWeaponTrait",
  Rarity = MatchesOne({ "Epic" })
})
requireAthenaFirst.C6.Exit.RoomName = "A_MiniBoss01"
requireAthenaFirst.C7.Room.UpgradeOptions = OneMatches({
  ItemName = "TriggerCurseTrait",
  Rarity = "Legendary"
})

RandomSynchronize()
local c2ExitRoomData = ChooseNextRoomData(CurrentRun)
local c2ExitDoor = {
  Room = ParasDoorPredictions.CreateRoom(CurrentRun, c2ExitRoomData, { SkipChooseReward = true, SkipChooseEncounter = true})
}
c2ExitDoor.Room.ChosenRewardType = ParasDoorPredictions.ChooseRoomReward(CurrentRun, c2ExitDoor.Room, "MetaProgress", {}, { PreviousRoom = C2Door.Room, Door = c2ExitDoor }) -- calls RandomSynchronize(4)
c2ExitDoor.Room.RewardStoreName = "MetaProgress"

CurrentRun.CurrentRoom = C2Door.Room

local results = FindIncrementally({
  SetupFindIncrementally(CurrentRun, c2ExitDoor, requireAresFirst, 2, 7, AthenaOffset),
  SetupFindIncrementally(CurrentRun, c2ExitDoor, requireAthenaFirst, 2, 7, AthenaOffset)
})

function Display(route)
  local display = {}
  for ci=1,50 do
    local thisRoom = route["C" .. ci]
    local nextRoom = route["C" .. (ci + 1)]

    if thisRoom ~= nil then
      local current = {}
      if thisRoom.UpgradeOptions ~= nil then
        local selected = SelectUpgrade(thisRoom.UpgradeOptions)
        current.Take = selected.ItemName .. " " .. selected.Rarity
      end
      if nextRoom ~= nil then
        current.Cast = nextRoom.Uses - thisRoom.oMinimum
        current.Door = thisRoom.Door.Room.ForceLootName or thisRoom.Door.Room.ChosenRewardType
      end
      display["C" .. ci] = current
    end
  end
  deep_print(display)
end

Display(results[1])