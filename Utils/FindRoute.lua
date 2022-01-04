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

function PredictRoomOptions( run, door, range )
  local oldUses = ParasDoorPredictions.CurrentUses
  local oldCurrentRun = CurrentRun
  CurrentRun = run
  local predictions = {}
  for uses=range.Min, range.Max do
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

function FindRemaining(run, door, requirements, i, results)
  local seeds = {}
  local cid = "C"..i
  local nextCid = "C"..(i+1)
  for _, reward in pairs(PredictRoomOptions(run, door, requirements[cid].Offset)) do
    table.insert(seeds, reward.Seed)
    if matches(requirements[cid].Room, reward) then
      results[cid] = reward
      if requirements[nextCid] then
        local run = MoveToNextRoom(run, reward, door)
        PickUpReward(run, requirements[cid].Boon)
        for _, door in pairs(ExitDoors(run, requirements[cid], reward)) do
          FindRemaining(run, door, requirements, i+1, results)
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
  for seed=requirements.Seed.Min,requirements.Seed.Max do
    if seed % 10000 == 0 then
      io.stderr:write(seed, "\n")
    end
    local c1_reward = PredictStartingRoomReward(seed)
    c1_reward.Seed = seed

    if matches(requirements.C1, c1_reward) then
      local run = CreateRun()
      PickUpReward(run) -- in C1
      RandomSynchronize(2) -- ChooseNextRoomData
      local c2_door = CreateDoor(
        c1_reward.SecondRoomName,
        c1_reward.SecondRoomReward,
        c1_reward.SecondRoomRewardStore)
      local result = { C1 = c1_reward }
      FindRemaining(run, c2_door, requirements, 2, result)
    end
  end
end
