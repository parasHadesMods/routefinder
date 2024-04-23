Import "Utils/FindRoute.lua"

local heroesExitDoor = {
  Room = CreateRoom(
    RoomData["C_PostBoss01"],
    {
      SkipChooseReward = true,
      SkipChooseEncounter = true
    }
  )
}

local offsets = PredictRoomOptions(
  CurrentRun,
  heroesExitDoor,
  { Min = 8000, Max = 12000 }
)

function hasTwistAndIchor(items)
  local has_ichor = one_matches({
    Name = "TemporaryMoveSpeedTrait"
  }, items)
  local has_twist = one_matches({
    Name = "RandomStoreItem"
  }, items)
  return has_ichor and has_twist
end

for i, result in pairs(offsets) do
  if hasTwistAndIchor(result.StoreOptions) then
    if hasTwistAndIchor(result.StoreOptionsReroll) then
      print(result.Uses)
      deep_print(result.StoreOptions)
      print("SlowReroll")
      deep_print(result.StoreOptionsReroll)
    elseif hasTwistAndIchor(result.StoreOptionsRerollFast) then
      print(result.Uses)
      deep_print(result.StoreOptions)
      print("FastReroll")
      deep_print(result.StoreOptionsRerollFast)
    end
  end
end
