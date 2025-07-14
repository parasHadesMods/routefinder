Import "Utils/FindRoute.lua"
Import "Utils/DeepPrint.lua"

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

local requirements = {
  C3 = {
    Offset = { Min = AthenaOffset, Max = AthenaOffset + 25 },
    Room = {},
    Exit = {
      Reward = "AresUpgrade",
    }
  },
  C4 = {
    Offset = { Min = 0, Max = 25, AddEstimatedOffset = true },
    Room = {
      UpgradeOptions = function(options)
        return one_matches({
            ItemName = "AresWeaponTrait",
            Rarity = function (rarity)
              return matches_one({ "Rare", "Epic" }, rarity)
            end
        }, options)
      end
    },
    Exit = {}
  },
  C5 = {
    Offset = { Min = 0, Max = 25, AddEstimatedOffset = true },
    Room = {},
    Exit = {
      Reward = "AthenaUpgrade"
    }
  },
  C6 = {
    Offset = { Min = 0, Max = 25, AddEstimatedOffset = true },
    Room = {
      UpgradeOptions = function(options)
        return one_matches({
            ItemName = "AthenaSecondaryTrait"
        }, options)
      end
    },
    Exit = {
      RoomName = "A_MiniBoss01"
    }
  },
  C7 = {
    Offset = { Min = 0, Max = 100, AddEstimatedOffset = true },
    Room = {
      UpgradeOptions = function(options)
        return one_matches({
            ItemName = "TriggerCurseTrait",
            Rarity = "Legendary"
        }, options)
      end
    },
    Exit = {}
  }
}


RandomSynchronize()
local c2ExitRoomData = ChooseNextRoomData(CurrentRun)
local c2ExitDoor = {
  Room = ParasDoorPredictions.CreateRoom(CurrentRun, c2ExitRoomData, { SkipChooseReward = true, SkipChooseEncounter = true})
}
c2ExitDoor.Room.ChosenRewardType = ParasDoorPredictions.ChooseRoomReward(CurrentRun, c2ExitDoor.Room, "MetaProgress", {}, { PreviousRoom = C2Door.Room, Door = c2ExitDoor }) -- calls RandomSynchronize(4)
c2ExitDoor.Room.RewardStoreName = "MetaProgress"

CurrentRun.CurrentRoom = C2Door.Room

local prediction = PredictLoot(c2ExitDoor) -- standing in front of c3 door, in c2
local result_table = {}
FindRemaining(CurrentRun, { c2ExitDoor }, requirements, 3, {}, result_table)
-- only need to show one result; for now pick the first one
local route = result_table[1]
local min_cost = nil
local min_display = nil
for _, route in pairs(result_table) do
  local display = {
    C2 = {
      Cast = route.C3.Uses - AthenaOffset,
      Door = c2ExitDoor.Room.ChosenRewardType
    },
    C3 = {
      Cast = route.C4.Uses - route.C3.EstimatedEndOfRoomOffset,
      Door = "AresUpgrade"
    },
    C4 = {
      Cast = route.C5.Uses - route.C4.EstimatedEndOfRoomOffset,
      Door = RewardForExitRoom(route.C4.Exits, route.C5.RoomName),
      Take = route.C4.UpgradeOptions[1].ItemName .. " " .. route.C4.UpgradeOptions[1].Rarity,
    },
    C5 = {
      Cast = route.C6.Uses - route.C5.EstimatedEndOfRoomOffset,
      Door = "AthenaUpgrade"
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