ParasDoorPredictions.Config.PrintNextSeed = false

Import "Utils/DeepPrint.lua"
Import "Utils/Matching.lua"

local function CreateRun()
  local oldCurrentRun = CurrentRun
  local run = StartNewRun()
  run.CurrentRoom.RewardStoreName = "RunProgress"
  CurrentRun = oldCurrentRun
  return run
end

local function EnterRoom(run, room)
  run.CurrentRoom = room
  run = RunWithUpdatedHistory(run)
  return run
end

local function CreateDoor( roomName, rewardType, rewardStore )
  local roomData = RoomData[roomName]
  local door = {
    Room = CreateRoom( roomData, { SkipChooseReward = true, SkipChooseEncounter = true } )
  }
  door.Room.ChosenRewardType = rewardType
  door.Room.RewardStoreName = rewardStore
  return door
end

function PredictC2Options( run, door )
  local oldUses = ParasDoorPredictions.CurrentUses
  local oldCurrentRun = CurrentRun
  CurrentRun = run
  local predictions = {}
  for uses=15,25 do
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
  "A_Combat10",
  "A_Combat14", -- Not actually small, just for testing
  "A_Combat15"  -- (trying to reproduce the charon% route)
}

local c1_requirements = {
  Type = "Hammer",
  SecondRoomRewardStore = "MetaProgress",
  FirstRoomChaos = false,
  SecondRoomChaos = false,
  SecondRoomName = function(roomName)
    return matches_one(small_rooms, roomName)
  end,
  HammerData = {
    Options = function(options)
      return one_matches({ Name = "GunExplodingSecondaryTrait"}, options)
    end
  }
}

local c2_exit_requirements = {
  Reward = "RoomRewardMoneyDrop",
  ChaosGate = true,
  Room = function(roomName)
    return matches_one(small_rooms, roomName)
  end
}

local c2_requirements = {
  Waves = 1,
  Enemies = function(enemies)
    return matches_table({"PunchingBagUnit"}, enemies)
  end,
  Exits = function(exits)
    return one_matches(c2_exit_requirements, exits)
  end
}

local c3_requirements = {
  Waves = 1,
  Enemies = function(enemies)
    return matches_table({"PunchingBagUnit"}, enemies)
  end
}

local seed = 19986
local c1_reward = PredictStartingRoomReward(seed)

if matches(c1_requirements, c1_reward) then
  local c2_matches = {}
  c1_reward.C2_Seeds = {} 
  local run = CreateRun()
  local door = CreateDoor(
    c1_reward.SecondRoomName,
    c1_reward.SecondRoomReward,
    c1_reward.SecondRoomRewardStore)
  for _, candidate in pairs(PredictC2Options(run, door)) do
    if matches(c2_requirements, candidate) then
      table.insert(c2_matches, candidate)
    end
   table.insert(c1_reward.C2_Seeds, candidate.Seed)
  end
  -- Move into C2
  local run = EnterRoom(run, door.Room)
  
  for _, c2_reward in pairs(c2_matches) do
    local c3_matches = {}
    for _, exit in pairs(filter(c2_exit_requirements, c2_reward.Exits)) do
      local door = CreateDoor(
        exit.Room,
        exit.Reward,
        "RunProgress" -- hard-coded for now
      )
      for _, candidate in pairs(PredictC2Options(run, door)) do
         if matches(c3_requirements, candidate) then
           deep_print(c2_reward)
           deep_print(candidate)
         end
      end
    end
  end
end

