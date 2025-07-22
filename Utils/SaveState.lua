Import "Utils/JsonPrint.lua"

local GameStateDenyList = {
    Cosmetics = true,
    LifetimeResourcesGained = true,
    EnemySpawns = true,
    EnemyDamage = true,
    EnemyKills = true,
    ItemInteractions = true,
    ScreensViewed = true,
    WeaponKills = true
}
local CurrentRunDenyList = {
    CurrentRoom = {
        CombatResolvedVoiceLines = true,
        EnterVoiceLines = true,
        ThreadedEvents = true,
        UnthreadedEvents = true,
        ThreadedEvents = true,
        InspectPoints = true,
        Encounter = {
            __any = true,
            Name = "__keep" -- keep _only_ the encounter name
        },
        SwapSounds = true,
        PreThingCreationUnthreadedEvents = true,
        ObjectStates = true
    },
    DamageRecord = true,
    GameplayTime = true,
    Hero = { 
        LastKillTime = true,
        Binks = true,
        InvulnerableFlags = true,
        HeroSurfaceDeathRumbleParameters = true,
        HeroTraitValuesCache = true,
        OutgoingDamageModifiers = true,
        PlayingSounds = true,
        PlayingVoiceLines = true,
        QueuedVoiceLines = true,
        RallyHealth = true,
        StoredAmmo = true,
        TraitAnimationAnchors = true,
        TraitDictionary = true,
        Traits = {
            __any = {
                __any = true,
                Name = "__keep" -- keep _only_ the trait name
            }
        },
        HitInvulnerableVoiceLines = true,
        LowHealthVoiceLines = true,
    },
    InvulnerableFlags = true,
    MoneyRecord = true,
    RoomHistory = {
        __any = {
            __any = true,
            Name = "__keep" -- keep _only_ the room name
        }
    },
    SpeechRecord = true,
    TriggerRecord = true,
    WeaponsFiredRecord = true,
}

local function deny(t, denyList)
    local copy = DeepCopyTable(t)
    for k,v in pairs(t) do
        local isDenied = denyList[k] or denyList.__any
        if type(isDenied) == "table" then
            copy[k] = deny(v, isDenied)
        elseif isDenied == "__keep" then
            -- do nothing
        elseif isDenied then
            copy[k] = nil
        end
    end
    return copy
end

function SaveState(state, filename)
    local json = JsonEncoder({
        CurrentRun = deny(state.CurrentRun, CurrentRunDenyList),
        GameState = deny(state.GameState, GameStateDenyList)
    })
    local outfile = io.open( filename, "w+" )
    outfile:write(json)
    outfile:close()
end