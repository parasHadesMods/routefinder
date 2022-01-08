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

function CreateC2Door( run, reward )
  local roomData = RoomData[reward.SecondRoomName]
  local door = {
    Room = CreateRoom( roomData, { SkipChooseReward = true, SkipChooseEncounter = true } )
  }
  if Contains(EllosBoonSelectorMod.BoonGods, reward.SecondRoomReward) then
    door.Room.ChosenRewardType = "Boon"
    door.Room.ForceLootName = reward.SecondRoomReward .. "Upgrade"
  else
    door.Room.ChosenRewardType = reward.SecondRoomReward
  end
  door.Room.RewardStoreName = reward.SecondRoomRewardStore
  return door
end

function UpdateRewardStoresForC2(run, reward)
  local rewardStore = run.RewardStores[reward.SecondRoomRewardStore]
  local firstRoomShrineReward = nil
  if reward.FirstRoomShrine then
    -- first remove the entry for the erebus gate
    local eligibleRewards = {}
    for key, candidate in pairs(rewardStore) do
      if IsSecondRoomRewardEligible(candidate.GameStateRequirements, reward.Type) and
         candidate.Name ~= "WeaponUpgrade" then -- Erebus gates can't have hammers
        table.insert(eligibleRewards, key)
      end
    end
    RandomSynchronize(4)
    local selectedKey = GetRandomValue( eligibleRewards )
    firstRoomShrineReward = rewardStore[selectedKey].Name
    rewardStore[selectedKey] = nil
  end
  -- then handle the normal exit
  local eligibleRewards = {}
  for key, candidate in pairs(rewardStore) do
    if IsSecondRoomRewardEligible(candidate.GameStateRequirements, reward.Type) and
       (reward.AllowDuplicates or candidate.Name ~= firstRoomShrineReward) then
      table.insert(eligibleRewards, key)
    end
  end
  RandomSynchronize(4)
  local selectedKey = GetRandomValue( eligibleRewards )
  rewardStore[selectedKey] = nil
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

function PickUpReward(run, requirements, reward)
  local lootName = run.CurrentRoom.ChosenRewardType
  if lootName == "LockKeyDropRunProgress" then
    run.NumRerolls = run.NumRerolls + 1
  end
  if lootName == "Boon" then
    lootName = run.CurrentRoom.ForceLootName
    local itemName = nil
    local rarity = nil
    if requirements == nil then
      -- no boon requirements, just pick the first option
      itemName = reward.UpgradeOptions[1].ItemName
      rarity = reward.UpgradeOptions[1].Rarity
    else
      itemName = requirements.ItemName
      rarity = requirements.Rarity or "Common"
    end
    local traitData = GetProcessedTraitData({ Unit = run.Hero, TraitName = itemName, Rarity = rarity })
    local trait = DeepCopyTable( traitData )
    table.insert(run.Hero.Traits, trait)
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
    local summary = {
      Seed = prediction.Seed,
      Uses = uses,
      StoreOptions = prediction.StoreOptions,
      HasCharonBag = prediction.HasCharonBag,
      UpgradeOptions = prediction.UpgradeOptions,
      UpgradeOptionsReroll = prediction.UpgradeOptionsReroll,
      Waves = 0,
      Enemies = {},
      Exits = {},
      Prediction = prediction
    }
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
        local exit = {
          RoomName = reward.RoomName,
          Room = reward.Room,
          ChaosGate = reward.ChaosGate,
          WellShop = reward.WellShop,
          Reward = reward.ForceLootName or reward.RewardType
        }
        table.insert(summary.Exits, exit)
      end
    end
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
    for k, exit in pairs(filter(room_requirements.Exit, reward.Exits)) do
      local door = {
        Room = DeepCopyTable(exit.Room)
      }
      table.insert(doors, door)
    end
  end
  return doors
end

function CheckForced(forcedSeed, seed)
  if forcedSeed == nil then
    return true
  else
    return forcedSeed == seed
  end
end

function FindRemaining(run, doors, requirements, i, results)
  local cid = "C"..i
  local nextCid = "C"..(i+1)
  -- Standing in front of a set of doors. Look at each door in turn.
  local seed = NextSeeds[1]
  for _, door in pairs(doors) do
    -- Predict what is behind each door; this depends on the rng offset.
    for _, reward in pairs(PredictRoomOptions(run, door, requirements[cid].Offset)) do
      if CheckForced(requirements[cid].ForcedSeed, reward.Seed) and matches(requirements[cid].Room, reward) then
        -- If we found a door that we like,
        results[cid] = reward
        if requirements[nextCid] then
          -- go through that door, pick up the reward, and find out what new doors we're presented with.
          local run = MoveToNextRoom(run, reward, door)
          PickUpReward(run, requirements[cid].Boon, reward)
          local doors = ExitDoors(run, requirements[cid], reward)
          -- Now we're standing in front of another set of doors.
          FindRemaining(run, doors, requirements, i+1, results)
        else
          -- or, if there are no more requirements, print the result and exit.
          for _, reward in pairs(results) do
            clean_reward(reward)
          end
          deep_print(results)
        end
        results[cid] = nil
      end
      NextSeeds[1] = seed -- rewind on return
    end
  end
end

function FindRoute(requirements)
  for seed=requirements.Seed.Min,requirements.Seed.Max do
    if seed % 10000 == 0 then
      io.stderr:write(seed, "\n")
    end
    local c1_reward = PredictStartingRoomReward(seed)
    c1_reward.Seed = seed

    if matches(requirements.C1, c1_reward) then
      local run = CreateRun()
      PickUpReward(run) -- in C1
      UpdateRewardStoresForC2(run, c1_reward)
      RandomSynchronize(2) -- ChooseNextRoomData
      local doors = { CreateC2Door(run, c1_reward) }
      local result = { C1 = c1_reward }
      FindRemaining(run, doors, requirements, 2, result)
    end
  end
end
