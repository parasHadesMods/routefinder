
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
    if prediction.Encounter.SpawnWaves then
      for i, wave in pairs(prediction.Encounter.SpawnWaves) do
        summary.Waves = summary.Waves + 1
        for j, spawn in pairs(wave.Spawns) do
          summary.Enemies[spawn.Name] = true
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
RouteFinderRoomReward = PredictStartingRoomReward(NextSeeds[1])
deep_print(RouteFinderRoomReward)

ScreenAnchors = {}
function GetIdsByType(args)
  if args.Name and args.Name == "HeroExit" then
    return { 1 }
  else
    print("Unexpected GetIdsByType")
  end
end
deep_print(PredictC2Options(RouteFinderRoomReward))

