Import "Utils/FindRoute.lua"

MinSeed = 15750000
MaxSeed = 15950000
MinC1Uses = 27
MaxC1Uses = 32
MinC3Uses = 5
MaxC3Uses = 8

local C2Door = CreateC2Door({
    SecondRoomName = "RoomSimple01", -- Athena Room
    SecondRoomReward = "Athena",
    SecondRoomRewardStore = "RunProgress"
})

local C1Run = CurrentRun
local C2Run = DeepCopyTable(CurrentRun)
local AthenaDashTrait = GetProcessedTraitData({ Unit = C2Run.Hero, TraitName = "AthenaRushTrait", Rarity = "Common" })
table.insert(C2Run.Hero.Traits, AthenaDashTrait)
C2Run.LootTypeHistory["AthenaUpgrade"] = 1

function ExitRewards(exitRewards)
  local rewards = {}
  for _, exit in pairs(exitRewards) do
    if exit.RewardType ~= "Boon" then
      table.insert(rewards, exit.RewardType)
    else
      table.insert(rewards, exit.ForceLootName)
    end
  end
  table.sort(rewards)
  return rewards
end

function EncounterWaves(encounter)
  local waves = {}
  for i, wave in pairs(encounter.SpawnWaves) do
    waves[i] = {}
    for _, spawn in pairs(wave.Spawns) do
      table.insert(waves[i], spawn.Name)
    end
    table.sort(waves[i])
  end
  return waves
end

function csv_entry(item)
  if type(item) == "table" then
    local entry = ""
    for _, i in pairs(item) do
      if entry ~= "" then
        entry = entry .. "+"
      end
      entry = entry .. i
    end
    return entry
  else
    return item
  end
end

function csv(row)
  local c = ""
  for _, item in pairs(row) do
    if c ~= "" then
      c = c .. ","
    end
    c = c .. csv_entry(item)
  end
  return c
end

function record(result)
  print(csv({
    result.C1_Seed,
    result.C2_Seed,
    result.C3_Seed,
    result.C4_Seed,
    result.C2_DoorRewards,
    result.C3_RoomName,
    result.C3_DoorRewards,
    result.C3_Waves[1] or "",
    result.C3_Waves[2] or "",
    result.C3_Waves[3] or "",
    result.C4_RewardChosen,
    result.C4_RoomName,
    result.C4_DoorRewards,
    result.C4_Waves[1] or "",
    result.C4_Waves[2] or "",
    result.C4_Waves[3] or ""
  }))
end

for seed=MinSeed,MaxSeed do
  if seed % 10000 == 0 then
    io.stderr:write(seed, "\n")
  end
  for uses=MinC1Uses,MaxC1Uses  do
    NextSeeds[1] = seed
    RandomSynchronize(uses)
    CurrentRun = C1Run
    local c2_prediction = PredictLoot(C2Door)
    local result = {
      C1_Seed = seed,
      C2_Seed = c2_prediction.Seed,
      C2_DoorRewards = ExitRewards(c2_prediction.NextExitRewards),
      C3_RoomName = c2_prediction.NextExitRewards[1].RoomName
    }
    local c3door = {
      Room = DeepCopyTable(c2_prediction.NextExitRewards[1].Room)
    }
    NextSeeds[1] = c2_prediction.Seed
    RandomSynchronize(6) -- uses at end of athena room
    CurrentRun = C2Run
    local c3_prediction = PredictLoot(c3door)
    result.C3_Seed = c3_prediction.Seed
    result.C3_DoorRewards = ExitRewards(c3_prediction.NextExitRewards)
    result.C3_Waves = EncounterWaves(c3_prediction.Encounter)
    for _, exit in pairs(c3_prediction.NextExitRewards) do
      result.C4_RoomName = exit.RoomName
      result.C4_RewardChosen = exit.ForceLootName or exit.RewardType
      local c4door = {
        Room = DeepCopyTable(exit.Room)
      }
      local reward = {
        Seed = c3_prediction.Seed,
        Prediction = c3_prediction
      }
      CurrentRun = MoveToNextRoom(C2Run, reward, c3door)
      PickUpReward(CurrentRun) -- c3 is always MetaProgress
      for uses=MinC3Uses,MaxC3Uses do
        NextSeeds[1] = c3_prediction.Seed
        RandomSynchronize(uses)
        local c4_prediction = PredictLoot(c4door)
        result.C4_Seed = c4_prediction.Seed
        result.C4_DoorRewards = ExitRewards(c4_prediction.NextExitRewards)
        result.C4_Waves = EncounterWaves(c4_prediction.Encounter)
        record(result)
      end
    end
  end
end
