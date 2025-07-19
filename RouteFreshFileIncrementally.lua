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

function RewardForExitRoom(exits, roomName)
  for _, exit in pairs(exits) do
    if exit.RoomName == roomName then
      return exit.Reward
    end
  end
end

function NewRequirements(cStart, cEnd)
  local r = {}
  for ci=cStart,cEnd do
    local cid = "C" .. ci
    r[cid] = { Room = {}, Exit = {} }
  end
  return r
end

local requireAresFirst = NewRequirements(3, 7)
requireAresFirst.C3.Exit.Reward = "AresUpgrade"
requireAresFirst.C4.Room.UpgradeOptions = OneMatches({
  ItemName = "AresWeaponTrait",
  Rarity = MatchesOne({ "Rare", "Epic" })
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
requireAthenaFirst.C3.Exit.Reward = "AthenaUpgrade"
requireAthenaFirst.C4.Room.UpgradeOptions = OneMatches({
  ItemName = "AthenaSecondaryTrait"
})
requireAthenaFirst.C5.Exit.Reward = "AresUpgrade"
requireAthenaFirst.C6.Room.UpgradeOptions = OneMatches({
  ItemName = "AresWeaponTrait",
  Rarity = MatchesOne({ "Rare", "Epic" })
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

local min_cost = nil
local min_display = nil
for _, route in pairs(results) do
  local display = {
    C2 = {
      Cast = route.C3.Uses - AthenaOffset,
      Door = c2ExitDoor.Room.ChosenRewardType
    },
    C3 = {
      Cast = route.C4.Uses - route.C3.EstimatedEndOfRoomOffset,
      Door = RewardForExitRoom(route.C3.Exits, route.C4.RoomName)
    },
    C4 = {
      Cast = route.C5.Uses - route.C4.EstimatedEndOfRoomOffset,
      Door = RewardForExitRoom(route.C4.Exits, route.C5.RoomName),
      Take = route.C4.UpgradeOptions[1].ItemName .. " " .. route.C4.UpgradeOptions[1].Rarity,
    },
    C5 = {
      Cast = route.C6.Uses - route.C5.EstimatedEndOfRoomOffset,
      Door = RewardForExitRoom(route.C5.Exits, route.C6.RoomName)
    },
    C6 = {
      Cast = route.C7.Uses - route.C6.EstimatedEndOfRoomOffset,
      Take = route.C6.UpgradeOptions[1].ItemName .. " " .. route.C6.UpgradeOptions[1].Rarity,
      Door = RewardForExitRoom(route.C6.Exits, route.C7.RoomName)
    },
    C7 = {
      Take = "TriggerCurseTrait"
    }
  }
  local cost = display.C2.Cast + display.C3.Cast + display.C4.Cast + display.C5.Cast + display.C6.Cast
  if min_cost == nil or min_cost > cost then
    min_cost = cost
    min_display = display
  end
end

if min_display ~= nil then
  deep_print(min_display)
end