RandomInit()

RouteFinderRoomReward = PredictStartingRoomReward(NextSeeds[1])

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

deep_print(RouteFinderRoomReward, 0)
