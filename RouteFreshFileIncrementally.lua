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

-- First section - we want to get Merciful end by C7
local requireAresFirst = NewRequirements(3, 7)
requireAresFirst.SelectUpgrade = SelectUpgrade
requireAresFirst.C3.Exit.Reward = "AresUpgrade"
requireAresFirst.C4.Room.UpgradeOptions = OneMatches({
  ItemName = "AresWeaponTrait",
  Rarity = "Epic"
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
  Rarity = "Epic"
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
  SetupFindIncrementally(CurrentRun, GameState, c2ExitDoor, requireAresFirst, 2, 7, AthenaSeed, AthenaOffset),
  SetupFindIncrementally(CurrentRun, GameState, c2ExitDoor, requireAthenaFirst, 2, 7, AthenaSeed, AthenaOffset)
})

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
      display[ci] = current
    end
  end
  deep_print(display)
end

local meRoute = results[1]
Display(meRoute)

-- Second section. All we care about is getting Impending Doom early, with low manips.
-- It might not be possible to get it before Meg if we haven't had a bag refill yet.
-- We also want to avoid midshop in Tartarus because it will probably throw us off route.
local secondSectionStates = {}
local basicRequirements = NewRequirements(8, 17)
basicRequirements.SelectUpgrade = SelectUpgrade
for ci=8,11 do
  basicRequirements["C"..ci].Exit.Reward = Not("Shop")
end
basicRequirements.C13.ExtraWiggleRoom = 1 -- Charon's groaning is sometimes unavoidable
basicRequirements.C15.ForceMinimumOffset = 12 -- a lot of yapping can occur here, but we can reset using the well
-- C8 and C9 are too early for another run reward
-- C13 = endshop, C14 = meg, C15 = stairs, C16 = asphodel intro, C17 = next chance to get impending
local possibleImpendingDoomRooms = { 10, 11, 12, 17 }
for _, ci in ipairs(possibleImpendingDoomRooms) do
  local requirements = DeepCopyTable(basicRequirements)
  requirements["C"..ci].Room.UpgradeOptions = OneMatches({
    ItemName = "AresLongCurseTrait"
  })
  local state = ResumeFindIncrementally(meRoute.C7, requirements, 7, 17, 1)
  table.insert(secondSectionStates, state)
end

local secondSectionResults = FindIncrementally(secondSectionStates)
local c13Route = secondSectionResults[1]
Display(c13Route)
secondSectionStates = nil -- don't need these anymore

-- Third section. All we care about is avoiding Dio and getting 2-sack.
local thirdSectionStates = {}
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
local thirdSectionResult = FindIncrementally({
  ResumeFindIncrementally(c13Route.C17, requirements, 17, 38, 1)
})
local thirdRoute = thirdSectionResult[1]
Display(thirdRoute)
thirdSectionStates = nil

-- Split out styx finding to reduce search space
local finalSectionStates = {}
local finalSectionResult = FindIncrementally({
  ResumeFindIncrementally(thirdRoute.C38, requirements, 38, 43, 1)
})
local finalRoute = finalSectionResult[1]
Display(finalRoute)