Import "Utils/FindRoute.lua"

local styxDoor = {
  Room = CreateRoom(
    RoomData["D_Hub"],
    {
      SkipChooseReward = true,
      SkipChooseEncounter = true
    }
  )
}
-- If you want to test alternative manips out of stairs
-- local StairsSeed = -1649347990

-- local alternativeSeeds = {}
-- NextSeeds[1] = StairsSeed
-- for i=12,100 do
--   RandomSynchronize(i)
--   local seed = RandomInt(-2147483647, 2147483646)
--   table.insert(alternativeSeeds, seed)
-- end

local requirements = {
  C39 = { -- Styx Hub
    Offset = { Min = 600, Max = 6000},
    Room = {
      --Seed = 1310035843
    },
    Exit = {
      StyxMiniBoss = true
    },
    SkipReward = true
  },
  C40 = { -- Tunnel 1
    -- Which Path adds extra increment, as does ... ???
    Offset = { Min = 2, Max = 2, AddEstimatedOffset = true },
    Room = {
      --Seed = 277685556
    },
    Exit = {},
    SkipReward = true
  },
  C41 = { -- Tunnel 2
    -- Extra offset from chaos curse expiring
    Offset = { Min = 1, Max = 1, AddEstimatedOffset = true },
    Room = {
      --Seed = 1833339984
    },
    Exit = {},
    SkipReward = true
  },
  C42 = { -- Tunnel 3
    -- Extra offset from another chaos curse expiring!
    Offset = { Min = 1, Max = 1, AddEstimatedOffset = true },
    Room = {
      --Seed = 1861540651,
      Exits = {
        {
          RoomName = "D_MiniBoss04" -- Bother
        }
      }
    },
    Exit = {},
    SkipReward = true
  },
}

FindRemaining(CurrentRun, { styxDoor }, requirements, 39, {})
