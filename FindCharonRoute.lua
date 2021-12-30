ParasDoorPredictions.Config.PrintNextSeed = false
if RngDisplayMod then
  RngDisplayMod.config.ShowSeed = false
  RngDisplayMod.config.ShowUses = false
end

Import "Utils/DeepPrint.lua"
Import "Utils/Matching.lua"

local function CreateRun()
  local oldCurrentRun = CurrentRun
  local run = StartNewRun()
  run.CurrentRoom.RewardStoreName = "RunProgress"
  CurrentRun = oldCurrentRun
  return run
end

local function CreateDoor( roomName, rewardType, rewardStore )
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

local function CreateSecretDoor( currentRun )
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

local function PickUpReward(run, requirements, chamber, reward)
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
  RoomName = function(roomName)
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

local c4_exit_requirements = {
  Reward = "AphroditeUpgrade",
  RoomName = "A_Reprieve01"
}

local c4_requirements = {
  Exits = function(exits)
    return one_matches(c4_exit_requirements, exits)
  end,
  Prediction = {
    UpgradeOptions = function(options)
      return one_matches({ SecondaryItemName = "ChaosCurseHealthTrait" }, options)
    end
  }
}

local c5_exit_requirements = {
  RoomName = "A_Shop01"
}

local c5_boon_requirements = {
  ItemName = "AphroditeShoutTrait"
}

local c5_requirements = {
  Exits = function(exits)
    return one_matches(c5_exit_requirements, exits)
  end,
  Prediction = {
    UpgradeOptionsReroll = function(reroll_options)
      return one_matches(c5_boon_requirements, reroll_options)
    end
  }
}

local c6_requirements = {
  Prediction = {
    HasCharonBag = true,
    StoreOptions = function(store_items)
      return one_matches({
        Name = "HermesUpgradeDrop",
        Args = {
          UpgradeOptions = function(options)
            return one_matches({
              Rarity = "Legendary"
            }, options)
          end
        }
      }, store_items)
    end
  }
}

function clean_reward(reward)
  reward.Prediction = nil
  for _, exit in pairs(reward.Exits) do
    exit.Room = nil
  end
end

DebugFalse=true
for seed=3181228,3181228 do
  if seed % 10000 == 0 then
    io.stderr:write(seed, "\n")
  end
  local c1_reward = PredictStartingRoomReward(seed)
  c1_reward.Seed = seed

  if matches(c1_requirements, c1_reward) then
    local c2_matches = {}
    c1_reward.C2_Seeds = {}
    local run = CreateRun()
    RandomSynchronize(2) -- ChooseNextRoomData
    local c2_door = CreateDoor(
      c1_reward.SecondRoomName,
      c1_reward.SecondRoomReward,
      c1_reward.SecondRoomRewardStore)
    for _, candidate in pairs(PredictRoomOptions(run, c2_door, 15, 25)) do
      if matches(c2_requirements, candidate) then
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
      for _, exit in pairs(filter(c2_exit_requirements, c2_reward.Exits)) do
        local c3_door = {
          Room = DeepCopyTable(exit.Room)
        }
        NextSeeds[1] = c2_reward.Seed
        for _, c3_reward in pairs(PredictRoomOptions(run, c3_door, 7, 17)) do
          if matches(c3_requirements, c3_reward) then
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
              if matches(c4_requirements, c4_reward) then
                c4_reward.UpgradeOptions = c4_reward.Prediction.UpgradeOptions
                -- Leave C3 and update history
                local run = RunWithUpdatedHistory(run)
                run.RewardStores = DeepCopyTable(c4_reward.Prediction.CurrentRun.RewardStores)
                run.LastWellShopDepth = c4_reward.Prediction.CurrentRun.LastWellShopDepth
                -- Enter C4 and pick up reward
                print("Enter C4")
                local c4 = DeepCopyTable(c4_door.Room)
                c4.Encounter = c4_reward.Prediction.Encounter
                run.CurrentRoom = c4
                PickUpReward(run, nil, "C4")
                for _, exit in pairs(filter(c4_exit_requirements, c4_reward.Exits)) do
                  local c5_door = {
                    Room = DeepCopyTable(exit.Room)
                  }
                  NextSeeds[1] = c4_reward.Seed
                  for _, c5_reward in pairs(PredictRoomOptions(run, c5_door, 6, 26)) do
                    if matches(c5_requirements, c5_reward) then
                      -- Leave c4 and update history
                      local run = RunWithUpdatedHistory(run)
                      run.RewardStores = DeepCopyTable(c5_reward.Prediction.CurrentRun.RewardStores)
                      run.LastWellShopDepth = c5_reward.Prediction.CurrentRun.LastWellShopDepth
                      -- Enter C5 and pick up reward
                      local c5 = DeepCopyTable(c5_door.Room)
                      c5.Encounter = c5_reward.Prediction.Encounter
                      run.CurrentRoom = c5
                      PickUpReward(run, c5_boon_requirements, "C5")
                      for _, exit in pairs(filter(c5_exit_requirements, c5_reward.Exits)) do
                        local c6_door = {
                          Room = DeepCopyTable(exit.Room)
                        }
                        NextSeeds[1] = c5_reward.Seed
                        for _, c6_reward in pairs(PredictRoomOptions(run, c6_door, 5, 25)) do
                          if matches(c6_requirements, c6_reward) then
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
