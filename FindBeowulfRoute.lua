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
  Seed = { Min = 588384, Max = 588384 },
  C1 = {  -- different format since Ello's is used for C1 instead of Para's
    Type = "Hammer",
    SecondRoomReward= "Poseidon", -- not PoseidonUpgrade
    FirstRoomChaos = false,
    SecondRoomChaos = true,
    HammerData = {
      Options = function(options)
        return one_matches({ Name = "ShieldRushProjectileTrait"}, options)
      end
    }
  },
  C2 = {
    Offset = { Min = 15, Max = 25 },
    ForcedSeed = 1123323008,
    Room = {
      UpgradeOptions = function(options)
        return one_matches(c2_boon, options)
      end
    },
    Boon = c2_boon,
    Exit = "SecretDoor"
  },
  C3 = {
    Offset = { Min = 5, Max = 25 },
    ForcedSeed = -1312722704,
    Exit = {}
  },
  C4 = {
    Offset = { Min = 5, Max = 30 },
    ForcedSeed = 444389298,
    Room = {},
    Exit = {
      WellShop = true
    }
  },
  C5 = {
    Offset = { Min = 5, Max = 30 },
    ForcedSeed = 1501741483,
    Room = {
      StoreOptions = function(options)
        return one_matches({ Name = "TemporaryForcedSecretDoorTrait"}, options)
      end
    },
    Exit = {}
  }
}

--DebugFalse=true
--ParasDoorPredictions.Config.PrintRngUses = true
FindRoute(requirements)
