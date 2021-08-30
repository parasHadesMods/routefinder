
function deep_print(t, indent)
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
  for uses=10,19 do
    RandomSynchronize(uses)
    predictions[uses] = PredictLoot(door)
  end 
  RandomSynchronize(oldUses) -- reset
  CurrentRun = oldCurrentRun
end

RandomInit()
RouteFinderRoomReward = PredictStartingRoomReward(NextSeeds[1])
deep_print(RouteFinderRoomReward, 0)

ScreenAnchors = {}
function GetIdsByType(args)
  if args.Name and args.Name == "HeroExit" then
    return { 1 }
  else
    print("Unexpected GetIdsByType")
  end
end
deep_print(PredictC2Options(RouteFinderRoomReward))

