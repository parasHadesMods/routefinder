Import "Utils/FindRoute.lua"
Import "Utils/LazyDeepCopyTable.lua"
DeepCopyTable = LazyDeepCopyTable

local styxDoor = {
  Room = CreateRoom(
    RoomData["D_Hub"],
    {
      SkipChooseReward = false,
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
    Offset = { Min = 10, Max = 100 },
    Room = {
    },
    Exit = {
      StyxMiniBoss = true
    },
    SkipReward = true
  },
  C40 = { -- Tunnel 1
    -- Which Path adds extra increment (skippable), as does ... something else
    Offset = { Min = 12, Max = 12, AddEstimatedOffset = false },
    Room = {
      -- Enemies = function(enemies)
      --   return not one_matches("Crawler" , enemies)
      -- end
    },
    Exit = {},
    SkipReward = true
  },
  C41 = { -- Tunnel 2
    -- Extra offset from chaos curse expiring
    Offset = { Min = 1, Max = 1, AddEstimatedOffset = true },
    Room = {
      -- Enemies = function(enemies)
      --   return not one_matches("Crawler" , enemies)
      -- end
    },
    Exit = {},
    SkipReward = true
  },
  C42 = { -- Tunnel 3
    -- Extra offset from another chaos curse expiring!
    Offset = { Min = 1, Max = 1, AddEstimatedOffset = true },
    Room = {
      -- Enemies = function(enemies)
      --   return not one_matches("Crawler" , enemies)
      -- end,
      -- Exits = {
      --   {
      --     RoomName = "D_MiniBoss04" -- Bother
      --   }
      -- }
    },
    Exit = {},
    SkipReward = true
  },
  C43 = { -- Miniboss
    Offset = { Min = 0, Max = 0, AddEstimatedOffset = true},
    Room = {},
    Exit = {},
  },
  C44 = { -- D_Hub
    Offset = { Min = 0, Max = 0, AddEstimatedOffset = true},
    Room = {},
    Exit = {},
    SkipReward = true
  },
  C45 = { -- Tunnel 2 # 1
    Offset = { Min = 33, Max = 33, AddEstimatedOffset = false},
    Room = {
      -- Enemies = function(enemies)
      --   return not one_matches("Crawler" , enemies)
      -- end
    },
    Exit = {},
    SkipReward = true
  },
  C46 = { -- Tunnel 2 # 2
    Offset = { Min = 0, Max = 0, AddEstimatedOffset = true},
    Room = {
      -- Enemies = function(enemies)
      --   return not one_matches("Crawler" , enemies)
      -- end
    },
    Exit = {},
    SkipReward = true
  },
  C47 = { -- Tunnel 2 # 3
    Offset = { Min = 0, Max = 0, AddEstimatedOffset = true},
    Room = {
      -- Enemies = function(enemies)
      --   return not one_matches("Crawler" , enemies)
      -- end,
      -- Exits = {
      --   {
      --     RoomName = "D_Reprieve01" -- Sack
      --   }
      -- }
    },
    Exit = {
    },
    SkipReward = true
  }
}

FindRemaining(CurrentRun, { styxDoor }, requirements, 39, {})
