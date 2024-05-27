Import "Utils/FindRoute.lua"

local megExitDoor = {
  Room = CreateRoom(
    RoomData["A_PostBoss01"],
    {
      SkipChooseReward = true,
      SkipChooseEncounter = true
    }
  )
}

--[[
  Normally, one of the items that you decline from the well
  (ie. do not buy) will be removed from the eligible pool,
  to ensure that you can't get exactly the same results the
  second time. However, for the purpose of trying to find
  a 4-ichor well with a single reroll, we need to buy both
  non-healing items before rerolling, so we're guaranteed
  to always have the same eligible list.
]]--
local EligibleStoreOptions = {
  {
    Name = "TemporaryImprovedWeaponTrait",
    Type = "Trait"
  },
  {
    Name = "TemporaryMoreAmmoTrait",
    Type = "Trait"
  },
  {
    Name = "TemporaryImprovedRangedTrait",
    Type = "Trait"
  },
  {
    Name = "TemporaryMoveSpeedTrait",
    Type = "Trait"
  },
  {
    Name = "TemporaryBoonRarityTrait",
    Type = "Trait"
  },
  {
    Name = "TemporaryArmorDamageTrait",
    Type = "Trait"
  },
  {
    Name = "TemporaryAlphaStrikeTrait",
    Type = "Trait"
  },
  {
    Name = "TemporaryBackstabTrait",
    Type = "Trait"
  },
  {
    Name = "TemporaryImprovedSecondaryTrait",
    Type = "Trait"
  },
  {
    Name = "TemporaryImprovedTrapDamageTrait",
    Type = "Trait"
  },
  {
    Name = "TemporaryForcedSecredDoorTrait",
    Type = "Trait"
  },
  {
    Name = "TemporaryForcedChallengeSwitch",
    Type = "Trait"
  },
  {
    Name = "TemporaryForcedFishingPointTrait",
    Type = "Trait"
  },
  {
    Name = "MetaDropRange",
    Type = "Consumable"
  },
  {
    Name = "GemDropRange",
    Type = "Consumable"
  },
  {
    Name = "KeepsakeChargeDrop",
    Type = "Consumable"
  },
  {
    Name = "RandomStoreItem",
    Type = "Consumable"
  }
}

function SimulateWellReroll(uses)
  RandomSynchronize(uses)
  local options = DeepCopyTable(EligibleStoreOptions)
  while TableLength( options ) > 2 do
    RemoveRandomValue( options )
  end
  return options
end

function hasTwistAndIchor(items)
  local has_ichor = one_matches({
    Name = "TemporaryMoveSpeedTrait"
  }, items)
  local has_twist = one_matches({
    Name = "RandomStoreItem"
  }, items)
  return has_ichor and has_twist
end

local EligibleTwistOptions = {
  {
    Name = "TemporaryWeaponLifeOnKillTrait",
    Type = "Trait"
  },
  {
    Name = "TemporaryDoorHealTrait",
    Type = "Trait"
  },
  {
    Name = "TemporaryImprovedWeaponTrait",
    Type = "Trait"
  },
  {
    Name = "TemporaryMoreAmmoTrait",
    Type = "Trait"
  },
  {
    Name = "TemporaryImprovedRangedTrait",
    Type = "Trait"
  },
  {
    Name = "TemporaryMoveSpeedTrait",
    Type = "Trait"
  },
  {
    Name = "TemporaryBoonRarityTrait",
    Type = "Trait"
  },
  {
    Name = "TemporaryArmorDamageTrait",
    Type = "Trait"
  },
  {
    Name = "TemporaryAlphaStrikeTrait",
    Type = "Trait"
  },
  {
    Name = "TemporaryBackstabTrait",
    Type = "Trait"
  },
  {
    Name = "TemporaryImprovedSecondaryTrait",
    Type = "Trait"
  },
  {
    Name = "TemporaryImprovedTrapDamageTrait",
    Type = "Trait"
  },
  {
    Name = "EmptyMaxHealthDrop",
    Type = "Trait"
  }
}

function SimulateFatefulTwist(uses)
  RandomSynchronize(uses)
  local run = DeepCopyTable(CurrentRun)
  TmpPlayedRandomLines = DeepCopyTable(PlayedRandomLines)
  TmpPlayingVoiceLines = {}
  TmpGlobalCooldowns = {}
  SimulateVoiceLines(run, GlobalVoiceLines.PurchasedWellShopItemVoiceLines)
  local randomItem = GetRandomValue( EligibleTwistOptions )
  return randomItem.Name
end

--local PreHeroesSeed = -1636193844

-- local alternativeSeeds = {}
-- NextSeeds[1] = PreHeroesSeed
-- for i=22,30 do
--   RandomSynchronize(i)
--   local seed = RandomInt(-2147483647, 2147483646)
--   table.insert(alternativeSeeds, seed)
-- end

for i, seed in pairs({ 1857589220 }) do
  NextSeeds[1] = seed
  print("Meg " .. seed)
  for i=0,30 do
    if i % 100 == 0 then
      print(i)
    end
    local result = PredictRoomOptions(
      CurrentRun,
      megExitDoor,
      { Min = i, Max = i})[1]
    if result.Seed == -2062222331 then -- hasTwistAndIchor(result.StoreOptions) then
      local oldSeed = NextSeeds[1]
      NextSeeds[1] = result.Seed
      for i=6,8 do
        local rerollResult = SimulateWellReroll(i)
        if true then -- hasTwistAndIchor(rerollResult) then
          deep_print({
            Seed = result.Seed,
            Uses = result.Uses,
            Well = result.StoreOptions
          })
          print("Reroll @ " .. i)
          deep_print(rerollResult)
          local twistOffset = 11
          repeat
            twistOffset = twistOffset + 1
          until SimulateFatefulTwist(twistOffset) == "TemporaryMoveSpeedTrait"

          print("Twist Offset " .. twistOffset)
        end
      end
      NextSeeds[1] = oldSeed
      RandomSynchronize()
    end
  end
end
