
function deep_print(t, indent)
  if not indent then indent = 0 end
  local indentString = ""
  for i = 1, indent do
    indentString = indentString .. "  "
  end
  for k,v in pairs(t) do
    if type(v) == "table" then
      print(indentString..k)
      deep_print(v, indent + 1)
    else
      print(indentString..k, v)
    end
  end
end

function matches(requirements, candidate)
  for k,v in pairs(requirements) do
    if candidate[k] == nil then
      return false
    end
    if type(v) == "function" then
      if not v(candidate[k]) then
        return false
      end
    elseif type(v) == "table" then
      if not matches(v, candidate[k]) then
        return false
      end
    elseif candidate[k] ~= v then
      return false
    end
  end
  return true
end

function one_matches(requirements, candidates)
  if type(candidates) ~= "table" then
    return false
  end
  for _,candidate in pairs(candidates) do
    if matches(requirements, candidate) then
      return true
    end
  end
  return false
end

function matches_table(requirements, candidates)
  if type(candidates) ~= "table" then
    return false
  end
  for k,v in pairs(candidates) do
    if requirements[k] == nil then
      return false
    end
    if requirements[k] ~= v then
      return false
    end
  end
  return true
end

function matches_one(options, candidate)
  for k,v in pairs(options) do
    if v == candidate then
      return true
    end
  end
  return false
end

function PredictC2Options( roomReward )
  local oldUses = ParasDoorPredictions.CurrentUses
  local oldCurrentRun = CurrentRun
  CurrentRun = StartNewRun()
  CurrentRun.CurrentRoom.RewardStoreName = "RunProgress" -- C1 is always gold laurel
  local roomData = RoomData[roomReward.SecondRoomName]
  local door = {
    Room = CreateRoom( roomData, { SkipChooseReward = true, SkipChooseEncounter = true } )
  }
  door.Room.ChosenRewardType = roomReward.SecondRoomReward
  door.Room.RewardStoreName = roomReward.SecondRoomRewardStore
  local predictions = {}
  for uses=10,25 do
    RandomSynchronize(uses)
    local prediction = PredictLoot(door)
    local summary = { Seed = prediction.Seed, Waves = 0, Enemies = {}, Exits = {} }
    local addedEnemy = {}
    if prediction.Encounter.SpawnWaves then
      for i, wave in pairs(prediction.Encounter.SpawnWaves) do
        summary.Waves = summary.Waves + 1
        for j, spawn in pairs(wave.Spawns) do
          if not addedEnemy[spawn.Name] then -- ensure uniqueness
            addedEnemy[spawn.Name] = true
            table.insert(summary.Enemies, spawn.Name)
          end
        end
      end
    end
    if prediction.NextExitRewards then
      for k, reward in pairs(prediction.NextExitRewards) do
        local exit = { Room = reward.RoomName }
        if reward.ForceLootName then
          exit.Reward = reward.ForceLootName
        else
          exit.Reward = reward.RewardType
        end
        exit.ChaosGate = reward.ChaosGate
        table.insert(summary.Exits, exit)
      end
    end
    summary.Uses = uses
    table.insert(predictions, summary)
  end
  RandomSynchronize(oldUses) -- reset
  CurrentRun = oldCurrentRun
  return predictions
end

RandomInit()
ScreenAnchors = {}
function GetIdsByType(args)
  if args.Name and args.Name == "HeroExit" then
    return { 1 }
  else
    print("Unexpected GetIdsByType")
  end
end

local small_rooms = {
  "A_Combat01",
  "A_Combat03",
  "A_Combat04",
  "A_Combat05",
  "A_Combat06",
  "A_Combat07",
  "A_Combat08A",
  "A_Combat09",
  "A_Combat10"
}

local c1_requirements = {
  Type = "Hammer",
  SecondRoomRewardStore = "MetaProgress",
  FirstRoomChaos = false,
  SecondRoomName = function(roomName)
    return matches_one(small_rooms, roomName)
  end,
  HammerData = {
    Options = function(options)
      return one_matches({ Name = "GunExplodingSecondaryTrait"}, options)
    end
  }
}

local c2_requirements = {
  Waves = 1,
  Enemies = function(enemies)
    return matches_table({"PunchingBagUnit"}, enemies)
  end,
  Exits = function(exits)
    return one_matches({Reward = "RoomRewardMoneyDrop", ChaosGate = true}, exits)
  end
}

for seed=19000,25000 do
  local c1_reward = PredictStartingRoomReward(seed)
  if matches(c1_requirements, c1_reward) then
    local c2_matches = {}
    for _, candidate in pairs(PredictC2Options(c1_reward)) do
      if matches(c2_requirements, candidate) then
        table.insert(c2_matches, candidate)
      end
    end
    if not IsEmpty(c2_matches) then
      print("Seed:", seed)
      deep_print(c1_reward, 1)
      for _, candidate in pairs(c2_matches) do
        deep_print(candidate, 1)
      end
    end
  end
end

