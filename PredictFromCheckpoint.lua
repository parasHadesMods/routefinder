Import "Utils/Checkpoint.lua"
Import "Utils/FindRoute.lua"
Import "Utils/FindIncrementally.lua"

local checkpoint = ReadCheckpoint(CheckpointFileName)
local room = checkpoint[CheckpointRoom]

print("==== CHECKPOINT ====")
print(NextSeeds[1], room.Seed)
NextSeeds[1] = room.Seed
for k,v in pairs(room.State.CurrentRun.RewardStores.RunProgress) do
    print(k, v.Name)
end
for _, reward in pairs(PredictRoomOptions(room.State, room.Door, { Min = PredictAtOffset, Max = PredictAtOffset })) do
    local store = reward.Prediction.CurrentRun.RewardStores.RunProgress
    local nextState = moveToNextRoom(room.State, reward, room.Door)
    clean_reward(reward)
    deep_print(reward)
    print("Prediction:")
    for k,v in pairs(store) do
        print(k, v.Name)
    end
    print("Next:")
    for k,v in pairs(nextState.CurrentRun.RewardStores.RunProgress) do
        print(k, v.Name)
    end
end

print("==== FROM SAVE ====")
print(NextSeeds[1], room.Seed)
NextSeeds[1] = room.Seed
for k,v in pairs(CurrentRun.RewardStores.RunProgress) do
    print(k, v.Name)
end
for _, reward in pairs(PredictRoomOptions({ CurrentRun = CurrentRun, GameState = GameState}, room.Door, { Min = PredictAtOffset, Max = PredictAtOffset })) do
    local store = reward.Prediction.CurrentRun.RewardStores.RunProgress
    clean_reward(reward)
    deep_print(reward)
    for k,v in pairs(store) do
        print(k, v.Name)
    end
end