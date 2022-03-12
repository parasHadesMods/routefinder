Import "Utils/FindRoute.lua"

local C2Door = CreateC2Door({
    SecondRoomName = "RoomSimple01", -- Athena Room
    SecondRoomReward = "Athena",
    SecondRoomRewardStore = "RunProgress"
})

local AthenaDashTrait = GetProcessedTraitData({ Unit = CurrentRun.Hero, TraitName = "AthenaRushTrait", Rarity = "Common" })
table.insert(CurrentRun.Hero.Traits, AthenaDashTrait)
CurrentRun.LootTypeHistory["AthenaUpgrade"] = 1

C3RoomName = A_Combat16
C4RoomName = A_Combat12
C3Reward   = RoomRewardMetaPointDrop
C4Reward   = StackUpgrade

RandomSynchronize(uses)
local c2_prediction = PredictLoot(C2Door)
local c3door = {
  Room = DeepCopyTable(c2_prediction.NextExitRewards[1].Room)
}
NextSeeds[1] = c2_prediction.Seed
RandomSynchronize(6) -- uses at end of athena room
local c3_prediction = PredictLoot(c3door)
result.C3_Seed = c3_prediction.Seed
result.C3_DoorRewards = ExitRewards(c3_prediction.NextExitRewards)
result.C3_Waves = EncounterWaves(c3_prediction.Encounter)
for _, exit in pairs(c3_prediction.NextExitRewards) do
  result.C4_RoomName = exit.RoomName
  result.C4_RewardChosen = exit.ForceLootName or exit.RewardType
  local c4door = {
    Room = DeepCopyTable(exit.Room)
  }
  CurrentRun = MoveToNextRoom(CurrentRun, reward, c3door)
  PickUpReward(CurrentRun) -- c3 is always MetaProgress
  for uses=MinC3Uses,MaxC3Uses do
    NextSeeds[1] = c3_prediction.Seed
    RandomSynchronize(uses)
    local c4_prediction = PredictLoot(c4door)
    result.C4_Seed = c4_prediction.Seed
    result.C4_DoorRewards = ExitRewards(c4_prediction.NextExitRewards)
    result.C4_Waves = EncounterWaves(c4_prediction.Encounter)
  end
end
