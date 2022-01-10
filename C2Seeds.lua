if RngDisplayMod then
  RngDisplayMod.config.ShowSeed = false
  RngDisplayMod.config.ShowUses = false
end
RandomInit()
NextSeeds[1] = 2296272
for uses=15,40 do
  RandomSynchronize(uses)
  local seed = RandomInt(-2147483647, 2147483646)
  print(seed)
end
