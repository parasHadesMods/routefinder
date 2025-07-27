Import "Utils/SaveState.lua"

local prefix = ""
if FilePrefix ~= nil then
    prefix = FilePrefix .. "_"
end

SaveState({ CurrentRun = CurrentRun, GameState = GameState}, "saves/" .. prefix .. "save_file.json")