Import "Utils/SaveState.lua"
Import "Utils/Checkpoint.lua"

local prefix = ""
if FilePrefix ~= nil then
    prefix = prefix .. FilePrefix .. "_"
end
if CheckpointRoom ~= nil then
    prefix = prefix .. CheckpointRoom .. "_"
end

SaveState({ CurrentRun = CurrentRun, GameState = GameState}, "saves/" .. prefix .. "save_file.json")

if CheckpointFileName ~= nil then
    local loaded = ReadCheckpoint(CheckpointFileName)
    if loaded == nil then return end
    SaveState({ CurrentRun = loaded[CheckpointRoom].State.CurrentRun, GameState = loaded[CheckpointRoom].State.GameState}, "saves/" .. prefix .. "checkpoint.json")
end