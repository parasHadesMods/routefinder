Import "Utils/FindRoute.lua"

local small_rooms = {
  "A_Combat01",
  "A_Combat03",
  "A_Combat04",
  "A_Combat05",
  "A_Combat06",
  "A_Combat07",
  "A_Combat08A",
  "A_Combat09",
  "A_Combat10"
}

local c2_boon = {
  ItemName = "PoseidonRangedTrait", -- Even for Beowulf, Poseidon doesn't use ShieldLoadAmmo_
  Rarity = "Epic"
}

local requirements = {
  Seed = { Min = 0, Max = 5000000 },
  C1 = {  -- different format since Ello's is used for C1 instead of Para's
    Type = "Hammer",
    SecondRoomReward= "Poseidon", -- not PoseidonUpgrade
    FirstRoomChaos = false,
    SecondRoomChaos = false, -- C3 chaos entrance / C4 chaos gives better combat
    HammerData = {
      Options = function(options)
        return one_matches({ Name = "ShieldRushProjectileTrait"}, options)
      end
    }
  },
  C2 = {
    Offset = { Min = 15, Max = 25 },
    Room = {
      Waves = 1,
      Enemies = function(enemies)
        return matches_table({"PunchingBagUnit"}, enemies)
      end,
      UpgradeOptions = function(options)
        return one_matches(c2_boon, options)
      end
    },
    Boon = c2_boon,
    Exit = {
      ChaosGate = true
    }
  },
  C3 = {
    Offset = { Min = 5, Max = 25 },
    Room = {
      Waves = 1,
      Enemies = function(enemies)
        return matches_table({"PunchingBagUnit"}, enemies)
      end,
    },
    Exit = "SecretDoor"
  },
  C4 = {
    Offset = { Min = 5, Max = 25 },
    Room = {
      UpgradeOptions = function(options)
        return one_matches({
            ItemName = "ChaosBlessingAmmoTrait"
        }, options)
      end
    },
    Exit = {
      WellShop = true
    }
  },
  C5 = {
    Offset = { Min = 5, Max = 25 },
    Room = {
      Waves = 1,
      StoreOptions = function(options)
        return one_matches({ Name = "TemporaryForcedSecretDoorTrait" }, options)
      end
    },
    Exit = { RoomName = "A_Shop01" }
  },
  C6 = {
    Offset = { Min = 10, Max = 35 },
    Room = {
      StoreOptions = function(store_options)
        return one_matches({
            Name = "HermesUpgradeDrop",
            Args = {
              UpgradeOptions = function(options)
                return one_matches({
                    ItemName = "BonusDashTrait",
                    Rarity = "Epic"
                }, options)
              end
            }
        }, store_options)
      end
    }
  }
}

--DebugFalse=true
--ParasDoorPredictions.Config.PrintRngUses = true
FindRoute(requirements)
