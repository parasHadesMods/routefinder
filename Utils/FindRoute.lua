ParasDoorPredictions.Config.PrintNextSeed = false
if RngDisplayMod then
  RngDisplayMod.config.ShowSeed = false
  RngDisplayMod.config.ShowUses = false
end

Import "Utils/DeepPrint.lua"
Import "Utils/Matching.lua"

function CreateRun()
  local oldCurrentRun = CurrentRun
  local run = StartNewRun()
  run.CurrentRoom.RewardStoreName = "RunProgress"
  CurrentRun = oldCurrentRun
  return run
end

function CreateDoor( roomName, rewardType, rewardStore )
  local roomData = RoomData[roomName]
  local door = {
    Room = CreateRoom( roomData, { SkipChooseReward = true, SkipChooseEncounter = true } )
  }
  if rewardType == "AphroditeUpgrade" then
    door.Room.ChosenRewardType = "Boon"
    door.Room.ForceLootName = rewardType
  else
    door.Room.ChosenRewardType = rewardType
  end
  door.Room.RewardStoreName = rewardStore
  return door
end

function CreateSecretDoor( currentRun )
  -- Based on HandleSecretSpawns
  local currentRoom = currentRun.CurrentRoom
  RandomSynchronize( 13 )

  local secretRoomData = ChooseNextRoomData( currentRun, { RoomDataSet = RoomSetData.Secrets } )
  local secretDoor = DeepCopyTable( ObstacleData.SecretDoor )
  secretDoor.HealthCost = GetSecretDoorCost()
  local secretRoom = CreateRoom( secretRoomData )
  secretDoor.Room = secretRoom -- AssignRoomToExitDoor
  secretDoor.OnUsedPresentationFunctionName = "SecretDoorUsedPresentation"
  currentRun.LastSecretDepth = GetRunDepth( currentRun )

  return secretDoor
end

function PickUpReward(run, requirements)
  local lootName = run.CurrentRoom.ChosenRewardType
  if lootName == "LockKeyDropRunProgress" then
    run.NumRerolls = run.NumRerolls + 1
  end
  if lootName == "Boon" then
    lootName = run.CurrentRoom.ForceLootName
    if requirements.ItemName ~= nil then
      local rarity = requirements.Rarity or "Common"
      local traitData = GetProcessedTraitData({ Unit = run.Hero, TraitName = requirements.ItemName, Rarity = rarity })
      local trait = DeepCopyTable( traitData )
      table.insert(run.Hero.Traits, trait)
    end
  end
  run.LootTypeHistory[lootName] = (run.LootTypeHistory[lootName] or 0) + 1
end

function PredictRoomOptions( run, door, minUses, maxUses)
  local oldUses = ParasDoorPredictions.CurrentUses
  local oldCurrentRun = CurrentRun
  CurrentRun = run
  local predictions = {}
  for uses=minUses,maxUses do
    RandomSynchronize(uses)
    local prediction = PredictLoot(door)
    local summary = { Seed = prediction.Seed, Waves = 0, Enemies = {}, Exits = {}, Prediction = prediction }
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
        local exit = { RoomName = reward.RoomName, Room = reward.Room }
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

function clean_reward(reward)
  if reward.Prediction then
    reward.StoreOptions = reward.Prediction.StoreOptions
    reward.HasCharonBag = reward.Prediction.HasCharonBag
    reward.UpgradeOptions = reward.Prediction.UpgradeOptions
    reward.UpgradeOptionsReroll = reward.Prediction.UpgradeOptionsReroll
    reward.Prediction = nil
  end
  if reward.Exits then
    for _, exit in pairs(reward.Exits) do
      exit.Room = nil
    end
  end
end

function MoveToNextRoom(previousRun, reward, door)
  -- Leave previous room and update history to reflect what happened
  local run = RunWithUpdatedHistory(previousRun)
  run.RewardStores = DeepCopyTable(reward.Prediction.CurrentRun.RewardStores)
  run.LastWellShopDepth = reward.Prediction.CurrentRun.LastWellShopDepth
  -- Enter next room and pick up reward
  local room = DeepCopyTable(door.Room)
  room.Encounter = reward.Prediction.Encounter
  run.CurrentRoom = room
  NextSeeds[1] = reward.Seed
  return run
end

function ExitDoors(run, room_requirements, reward)
  local doors = {}
  if room_requirements.Exit == "SecretDoor" then
    table.insert(doors, CreateSecretDoor(run))
  else
    for _, exit in pairs(filter(room_requirements.Exit, reward.Exits)) do
      local door = {
        Room = DeepCopyTable(exit.Room)
      }
      table.insert(doors, door)
    end
  end
  return doors
end

local NextCid = {
  C1 = "C2",
  C2 = "C3",
  C3 = "C4",
  C4 = "C5",
  C5 = "C6",
  C6 = "C7"
}

local Ranges = {
  C2 = { Min = 15, Max = 25 },
  C3 = { Min = 7,  Max = 17 },
  C4 = { Min = 5,  Max = 25 },
  C5 = { Min = 6,  Max = 26 },
  C6 = { Min = 5,  Max = 25 }
}

function FindRemaining(run, door, requirements, cid, results)
  for _, reward in pairs(PredictRoomOptions(run, door, Ranges[cid].Min, Ranges[cid].Max)) do
    if matches(requirements[cid].Room, reward) then
      local nextCid = NextCid[cid]
      results[cid] = reward
      if requirements[nextCid] then
        local run = MoveToNextRoom(run, reward, door)
        PickUpReward(run, requirements[cid].Boon)
        for _, door in pairs(ExitDoors(run, requirements[cid], reward)) do
          FindRemaining(run, door, requirements, nextCid, results)
        end
      else
        for _, reward in pairs(results) do
          clean_reward(reward)
        end
        deep_print(results)
      end
      results[cid] = nil
    end
  end
end

function FindRoute(requirements)
--DebugFalse=true
for seed=2323902,2323902 do
  if seed % 10000 == 0 then
    io.stderr:write(seed, "\n")
  end
  local c1_reward = PredictStartingRoomReward(seed)
  c1_reward.Seed = seed

  if matches(requirements.C1, c1_reward) then
    local c2_matches = {}
    c1_reward.C2_Seeds = {}
    local run = CreateRun()
    RandomSynchronize(2) -- ChooseNextRoomData
    local c2_door = CreateDoor(
      c1_reward.SecondRoomName,
      c1_reward.SecondRoomReward,
      c1_reward.SecondRoomRewardStore)
    for _, candidate in pairs(PredictRoomOptions(run, c2_door, 15, 25)) do
      if matches(requirements.C2.Room, candidate) then
        table.insert(c2_matches, candidate)
      end
     table.insert(c1_reward.C2_Seeds, candidate.Seed)
    end
    PickUpReward(run) -- in C1
    for _, c2_reward in pairs(c2_matches) do
      local run = MoveToNextRoom(run, c2_reward, c2_door)
      PickUpReward(run, requirements.C2.Boon)
      for _, c3_door in pairs(ExitDoors(run, requirements.C2, c2_reward)) do
        for _, c3_reward in pairs(PredictRoomOptions(run, c3_door, 7, 17)) do
          if matches(requirements.C3.Room, c3_reward) then
            local run = MoveToNextRoom(run, c3_reward, c3_door)
            PickUpReward(run, requirements.C3.Boon)
            local c4_door = CreateSecretDoor( run ) -- hard-coded, need some way to indicate
            local result = {
              C1 = c1_reward,
              C2 = c2_reward,
              C3 = c3_reward
            }
            FindRemaining(run, c4_door, requirements, "C4", result)
          end
        end
      end
    end
  end
end
end
