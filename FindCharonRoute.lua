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

local c5_boon = {
  ItemName = "AphroditeShoutTrait"
}

local requirements = {
  Seed = { Min = 2323902, Max = 2323902 },
  C1 = {  -- different format due to Ello's
    Type = "Hammer",
    SecondRoomRewardStore = "MetaProgress",
    FirstRoomChaos = false,
    SecondRoomChaos = false,
    SecondRoomName = function(roomName)
      return matches_one(small_rooms, roomName)
    end,
    HammerData = {
      Options = function(options)
        return one_matches({ Name = "GunExplodingSecondaryTrait"}, options)
      end
    }
  },
  C2 = {
    Offset = { Min = 15, Max = 25 },
    Room = {
      Waves = 1,
      Enemies = function(enemies)
        return matches_table({"PunchingBagUnit"}, enemies)
      end
    },
    Exit = {
      Reward = "RoomRewardMoneyDrop",
      ChaosGate = true,
      RoomName = function(roomName)
        return matches_one(small_rooms, roomName)
      end
    }
  },
  C3 = {
    Offset = { Min = 7, Max = 17 },
    Room = {
      Waves = 1,
      Enemies = function(enemies)
        return matches_table({"PunchingBagUnit"}, enemies)
      end
    },
    Exit = "SecretDoor" -- special value
  },
  C4 = {
    Offset = { Min = 5, Max = 25 },
    Room = {
      UpgradeOptions = function(options)
        return one_matches({ SecondaryItemName = "ChaosCurseHealthTrait" }, options)
      end
    },
    Exit = {
      Reward = "AphroditeUpgrade",
      RoomName = "A_Reprieve01"
    }
  },
  C5 = {
    Offset = { Min = 6, Max = 26 },
    Room = {
      UpgradeOptionsReroll = function(reroll_options)
        return one_matches(c5_boon, reroll_options)
      end
    },
    Boon = c5_boon,
    Exit = { RoomName = "A_Shop01" }
  },
  C6 = {
    Offset = { Min = 5, Max = 25 },
    Room = {
      HasCharonBag = true,
      StoreOptions = function(store_items)
        return one_matches({
            Name = "HermesUpgradeDrop",
            Args = {
              UpgradeOptions = function(options)
                return one_matches({
                    Rarity = "Legendary"
                                   }, options)
              end
            }
        }, store_items)
      end
    }
  }
}

FindRoute(requirements)
