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
  reward.Prediction = nil
  for _, exit in pairs(reward.Exits) do
    exit.Room = nil
  end
end

function MoveToNextRoom(previousRun, prediction, door)
  -- Leave previous room and update history to reflect what happened
  local run = RunWithUpdatedHistory(previousRun)
  run.RewardStores = DeepCopyTable(prediction.CurrentRun.RewardStores)
  run.LastWellShopDepth = prediction.CurrentRun.LastWellShopDepth
  -- Enter next room and pick up reward
  local room = DeepCopyTable(door.Room)
  room.Encounter = prediction.Encounter
  run.CurrentRoom = room
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
    for _, c2_reward in pairs(c2_matches) do
      -- Leave C1 and update history to reflect what happened
      PickUpReward(run)
      local run = RunWithUpdatedHistory(run)
      run.RewardStores = DeepCopyTable(c2_reward.Prediction.CurrentRun.RewardStores)
      run.LastWellShopDepth = c2_reward.Prediction.CurrentRun.LastWellShopDepth
      -- Enter C2 and pick up reward
      local c2 = DeepCopyTable(c2_door.Room)
      c2.Encounter = c2_reward.Prediction.Encounter
      run.CurrentRoom = c2
      PickUpReward(run, nil, "C2")
      for _, exit in pairs(filter(requirements.C2.Exit, c2_reward.Exits)) do
        local c3_door = {
          Room = DeepCopyTable(exit.Room)
        }
        NextSeeds[1] = c2_reward.Seed
        for _, c3_reward in pairs(PredictRoomOptions(run, c3_door, 7, 17)) do
          if matches(requirements.C3.Room, c3_reward) then
            -- Leave C2 and update history
            local run = RunWithUpdatedHistory(run)
            run.RewardStores = DeepCopyTable(c3_reward.Prediction.CurrentRun.RewardStores)
            run.LastWellShopDepth = c3_reward.Prediction.CurrentRun.LastWellShopDepth
            -- Enter C3 and pick up reward
            local c3 = DeepCopyTable(c3_door.Room)
            c3.Encounter = c3_reward.Prediction.Encounter
            run.CurrentRoom = c3
            PickUpReward(run, nil, "C3", c3_reward)
            NextSeeds[1] = c3_reward.Seed
            local c4_door = CreateSecretDoor( run ) -- hard-coded, need some way to indicate
            for _, c4_reward in pairs(PredictRoomOptions(run, c4_door, 5, 25)) do
              if matches(requirements.C4.Room, c4_reward) then
                c4_reward.UpgradeOptions = c4_reward.Prediction.UpgradeOptions
                local run = MoveToNextRoom(run, c3_reward.Prediction, c4_door)
                PickUpReward(run, requirements.C4.Boon)
                for _, c5_door in pairs(ExitDoors(run, requirements.C4, c4_reward)) do
                  NextSeeds[1] = c4_reward.Seed
                  for _, c5_reward in pairs(PredictRoomOptions(run, c5_door, 6, 26)) do
                    if matches(requirements.C5.Room, c5_reward) then
                      local run = MoveToNextRoom(run, c4_reward.Prediction, c5_door)
                      PickUpReward(run, requirements.C5.Boon)
                      for _, c6_door in pairs(ExitDoors(run, requirements.C5, c5_reward)) do
                        NextSeeds[1] = c5_reward.Seed
                        for _, c6_reward in pairs(PredictRoomOptions(run, c6_door, 5, 25)) do
                          if matches(requirements.C6.Room, c6_reward) then
                            clean_reward(c2_reward)
                            clean_reward(c3_reward)
                            clean_reward(c4_reward)
                            clean_reward(c5_reward)
                            c6_reward.StoreOptions = c6_reward.Prediction.StoreOptions
                            c6_reward.HasCharonBag = c6_reward.Prediction.HasCharonBag
                            clean_reward(c6_reward)
                            deep_print({ C1 = c1_reward})
                            deep_print({ C2 = c2_reward})
                            deep_print({ C3 = c3_reward})
                            deep_print({ C4 = c4_reward})
                            deep_print({ C5 = c5_reward})
                            deep_print({ C6 = c6_reward})
                          end
                        end
                      end
                    end
                  end
                end
              end
            end
          end
        end
      end
    end
  end
end
end
