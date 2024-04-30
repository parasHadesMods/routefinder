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

print(NextSeeds[1])
local requirements = {
  C39 = { -- Styx Hub
    Offset = { Min = 41, Max = 41},
    Room = {
      Seed = 1300018164
    },
    Exit = {
      StyxMiniBoss = true
    },
    SkipReward = true
  },
  C40 = { -- Tunnel 1
    -- Which Path adds extra increment (skippable), as does ... something else
    Offset = { Min = 16, Max = 16, AddEstimatedOffset = false },
    Room = {
      Seed = 26912063
    },
    Exit = {},
    SkipReward = true
  },
  C41 = { -- Tunnel 2
    -- Extra offset from chaos curse expiring
    Offset = { Min = 13, Max = 13, AddEstimatedOffset = false },
    Room = {
      Seed = 736107039
    },
    Exit = {},
    SkipReward = true
  },
  C42 = { -- Tunnel 3
    -- Extra offset from another chaos curse expiring!
    Offset = { Min = 18, Max = 18, AddEstimatedOffset = false },
    Room = {
      Seed = 1216909903,
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
