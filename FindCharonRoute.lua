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

local c1_requirements = {
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
}

local c2_exit_requirements = {
  Reward = "RoomRewardMoneyDrop",
  ChaosGate = true,
  RoomName = function(roomName)
    return matches_one(small_rooms, roomName)
  end
}

local c2_requirements = {
  Waves = 1,
  Enemies = function(enemies)
    return matches_table({"PunchingBagUnit"}, enemies)
  end,
  Exits = function(exits)
    return one_matches(c2_exit_requirements, exits)
  end
}

local c3_requirements = {
  Waves = 1,
  Enemies = function(enemies)
    return matches_table({"PunchingBagUnit"}, enemies)
  end
}

local c4_exit_requirements = {
  Reward = "AphroditeUpgrade",
  RoomName = "A_Reprieve01"
}

local c4_requirements = {
  Exits = function(exits)
    return one_matches(c4_exit_requirements, exits)
  end,
  Prediction = {
    UpgradeOptions = function(options)
      return one_matches({ SecondaryItemName = "ChaosCurseHealthTrait" }, options)
    end
  }
}

local c5_exit_requirements = {
  RoomName = "A_Shop01"
}

local c5_boon_requirements = {
  ItemName = "AphroditeShoutTrait"
}

local c5_requirements = {
  Exits = function(exits)
    return one_matches(c5_exit_requirements, exits)
  end,
  Prediction = {
    UpgradeOptionsReroll = function(reroll_options)
      return one_matches(c5_boon_requirements, reroll_options)
    end
  }
}

local c6_requirements = {
  Prediction = {
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

local requirements = {
  Seed = { Min = 2323902, Max = 2323902 },
  C1 = c1_requirements, -- different format due to Ello's
  C2 = {
    Offset = { Min = 15, Max = 25 },
    Room = c2_requirements,
    Exit = c2_exit_requirements
  },
  C3 = {
    Offset = { Min = 7, Max = 17 },
    Room = c3_requirements,
    Exit = "SecretDoor" -- not requirements format?
  },
  C4 = {
    Offset = { Min = 5, Max = 25 },
    Room = c4_requirements,
    Exit = c4_exit_requirements
  },
  C5 = {
    Offset = { Min = 6, Max = 26 },
    Room = c5_requirements,
    Boon = c5_boon_requirements,
    Exit = c5_exit_requirements
  },
  C6 = {
    Offset = { Min = 5, Max = 25 },
    Room = c6_requirements
  }
}

FindRoute(requirements)
