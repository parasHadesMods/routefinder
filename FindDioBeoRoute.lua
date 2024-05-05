Import "Utils/FindRoute.lua"
Import "Utils/LazyDeepCopyTable.lua"
Import "Utils/pepperfish.lua"
DeepCopyTable = LazyDeepCopyTable

print( "Beginning search at timestamp " .. os.date("%Y-%m-%d %H:%M") )

local smallRooms = {
  "A_Combat01",
  "A_Combat03",
  "A_Combat04",
  "A_Combat05",
  "A_Combat06",
  "A_Combat07",
  "A_Combat08A",
  "A_Combat09",
  "A_Combat10",
}

local goodTartEnemies = {
    "PunchingBagUnit",
    "PunchingBagUnitElite",
    "ThiefMineLayerElite",
    "HeavyMeleeElite",
    "HeavyRangedElite",
}

local acceptableChaos = {
    "ChaosCurseSecondaryAttackTrait",
    "ChaosCurseDeathWeaponTrait",
    "ChaosCurseHiddenRoomReward",
    "ChaosCurseDamageTrait",
    "ChaosCurseTrapDamageTrait",
    "ChaosCurseMoveSpeedTrait",
    "ChaosCurseDashRangeTrait",
}

local usefulWellItems = {
    "TemporaryForcedSecretDoorTrait",
    "TemporaryMoreAmmoTrait",
    "TemporaryMoveSpeedTrait",
    "TemporaryImprovedRangedTrait",
    "RandomStoreItem",
}

local c2Boon = {
  ItemName = "DionysusRangedTrait", -- Even for Beowulf, DionysusRangedTrait doesn't use ShieldLoadAmmo_
  Rarity = "Epic",
}

local requirements = {
    Seed = { Min = 447000000, Max = 447100000 },
    C1 = {
        Type = "Hammer",
        SecondRoomReward = "Dionysus",
        FirstRoomChaos = false,
        SecondRoomChaos = false, -- C3 chaos entrance / C4 chaos gives better combat
        SecondRoomName = function( roomName ) return matches_one( smallRooms, roomName ) end,
        HammerData = {
            Options = function( options ) return one_matches( { Name = "ShieldRushProjectileTrait" }, options ) end
        },
    },
    C2 = {
        Offset = { Min = 15, Max = 25 },
        Room = {
            Waves = 1,
            Enemies = function( enemies )
                return any_matchers( goodTartEnemies, enemies )
            end,
            UpgradeOptions = function( options )
                return one_matches( c2Boon, options )
            end,
        },
        Boon = c2Boon,
        Exit = {
            RoomName = function( roomName ) return matches_one( smallRooms, roomName ) end,
            ChaosGate = true,
        }
    },
    C3 = {
        Offset = { Min = 5, Max = 25 },
        Room = {
            Waves = 1,
            Enemies = function( enemies )
                return any_matchers( goodTartEnemies, enemies )
            end,
        },
        Exit = "SecretDoor",
    },
    C4 = {
        Offset = { Min = 9, Max = 25 },
        Room = {
            UpgradeOptions = function( options )
                return one_matches(
                    {
                        ItemName = "ChaosBlessingAmmoTrait",
                        SecondaryItemName = function( curse ) return matches_one( acceptableChaos, curse ) end,
                    },
                    options
                )
            end
        },
        Exit = {
            RoomName = function( roomName ) return matches_one( smallRooms, roomName ) end,
            WellShop = true,
        },
    },
    C5 = {
        Offset = { Min = 5, Max = 25 },
        Room = {
            Waves = 1,
            Enemies = function( enemies )
                return any_matchers( goodTartEnemies, enemies )
            end,
            StoreOptions = function( options )
                return minimum_matches(
                    { Name = function( itemName ) return matches_one( usefulWellItems, itemName ) end },
                    options,
                    2
                )
            end
        },
        Exit = { RoomName = "A_Shop01" }
    },
    C6 = {
        Offset = { Min = 10, Max = 35 },
        Room = {
            StoreOptions = function( storeOptions )
            return one_matches({
                Name = "HermesUpgradeDrop",
                Args = {
                    UpgradeOptions = function(options)
                        return one_matches(
                            {
                                ItemName = "RushSpeedBoostTrait",
                                Rarity = "Epic"
                            },
                        options )
                    end
                }
            }, storeOptions )
            end
        },
        --[[Exit = {
            RoomName = function( roomName ) return matches_one( { "A_Story01", "A_MiniBoss03", "A_MiniBoss04" }, roomName ) end,
        },]]
    },
    C7 = {
        Offset = { Min = 5, Max = 5 },
        Room = {},
        Exit = {},
    },
}

print( "Checking seeds "..requirements.Seed.Min .. " to " .. requirements.Seed.Max )

profiler = newProfiler()
profiler:start()
-- look for routes 
FindRoute( requirements )
profiler:stop()
local outfile = io.open( "lua_profile.txt", "w+" )
profiler:report( outfile )
outfile:close()

print( "Finished search at timestamp " .. os.date("%Y-%m-%d %H:%M") )
