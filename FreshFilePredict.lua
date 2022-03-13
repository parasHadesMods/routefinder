Import "Utils/FindRoute.lua"
Import "Utils/JsonRead.lua"
Import "Utils/DeepPrint.lua"

local C2Door = CreateC2Door({
    SecondRoomName = "RoomSimple01", -- Athena Room
    SecondRoomReward = "Athena",
    SecondRoomRewardStore = "RunProgress"
})

local AthenaDashTrait = GetProcessedTraitData({ Unit = CurrentRun.Hero, TraitName = "AthenaRushTrait", Rarity = "Common" })
table.insert(CurrentRun.Hero.Traits, AthenaDashTrait)
CurrentRun.LootTypeHistory["AthenaUpgrade"] = 1

local function read_file(path)
  local file = io.open(path, "rb") -- r read mode and b binary mode
  if not file then return nil end
  local content = file:read "*a" -- *a or *all reads the whole file
  file:close()
  return content
end

UseRange = {
  Min = 0,
  Max = 100
}

RunToMatch = JsonDecoder(read_file("../seedfinder/run_for_prediction.json"))

for uses=UseRange.Min,UseRange.Max do
  NextSeeds[1] = RunToMatch.C1_Seed
  RandomSynchronize(uses)
  local prediction = PredictLoot(C2Door)
  if prediction.Seed == RunToMatch.C2_Seed then
    local c2_exit_door = {
      Room = DeepCopyTable(prediction.NextExitRewards[1].Room) -- one exit
    }
    NextSeeds[1] = prediction.Seed
    RandomSynchronize(6) -- offset at end of athena room
    local prediction = PredictLoot(c2_exit_door) -- standing in front of c3 door, in c2
    if prediction.Seed == RunToMatch.C3_Seed then
      for _, exit in pairs(prediction.NextExitRewards) do
        local c3_exit_reward = exit.ForceLootName or exit.RewardType
        if c3_exit_reward == RunToMatch.C3_Exit_Chosen then
          local c3_exit_door = { -- we already know where we're going to go before we enter c3
            Room = DeepCopyTable(exit.Room)
          }
          CurrentRun = MoveToNextRoom(CurrentRun, { Prediction = prediction }, c2_exit_door) -- c2 -> c3
          PickUpReward(CurrentRun) -- always metaprogress
          for uses = UseRange.Min,UseRange.Max do -- now ready to predict c4 since we have picked up c3 reward
            NextSeeds[1] = prediction.Seed
            RandomSynchronize(uses)
            local prediction = PredictLoot(c3_exit_door) -- standing in front of c4 doors, in c3
            if prediction.Seed == RunToMatch.C4_Seed then
              EstimatedOffset = prediction.EstimatedEndOfRoomOffset
              CurrentRun = MoveToNextRoom(CurrentRun, { Prediction = prediction }, c3_exit_door) -- c3 -> c4
              PickUpReward(CurrentRun, nil, prediction)
              local summary = {}
              for _, exit in pairs(prediction.NextExitRewards) do
                local c4_exit_reward = exit.ForceLootName or exit.RewardType
                summary[c4_exit_reward] = {}
                local door = {
                  Room = DeepCopyTable(exit.Room)
                }
                NextSeeds[1] = prediction.Seed
                local options = PredictRoomOptions(CurrentRun, door, { Min = EstimatedOffset, Max = EstimatedOffset + 7 })
                for _, option in pairs(options) do
                  clean_reward(option)
                  local summary_option = ""
                  for k,exit in pairs(option.Exits) do
                    if summary_option ~= "" then
                      summary_option = summary_option .. "+"
                    end
                    summary_option = summary_option .. exit.Reward
                  end
                  summary[c4_exit_reward][option.Uses] = summary_option
                end
              end
              deep_print(summary)
              return -- once we've found and printed the options, we're done
            end
          end
        end
      end
    end
  end
end
