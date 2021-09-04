-- Callbacks from the engine that we will never call;
-- these can just be nops.
local nop_functions = {
  "Using",
  "OnPreThingCreation",
  "OnAnyLoad",
  "OnUsed",
  "OnActivationFinished",
  "OnAutoUseFailed",
  "OnMenuOpened",
  "OnMenuCloseFinished",
  "OnPlayerMoveStarted",
  "OnPlayerMoveStopped",
  "OnControlPressed",
  "OnActiveUseTarget",
  "OnActiveUseTargetLost",
  "OnMouseOver",
  "OnMouseOff",
  "OnControlHotSwap",
  "OnMusicMarker",
  "OnKeyPressed",
  "OnWeaponFired",
  "OnWeaponTriggerRelease",
  "OnComeToRest",
  "OnRamWeaponComplete",
  "OnWeaponCharging",
  "OnWeaponChargeCanceled",
  "OnWeaponFailedToFire",
  "OnPerfectChargeWindowEntered",
  "OnHit",
  "OnProjectileReflect",
  "OnProjectileBlock",
  "OnProjectileDeath",
  "OnDodge",
  "OnSpawn",
  "OnHealed",
  "OnCollisionReaction",
  "OnCollisionEnd",
  "OnObstacleCollision",
  "OnUnitCollision",
  "OnMovementReaction",
  "OnAllegianceFlip",
  "OnTouchdown",
  "OnEffectApply",
  "OnEffectCleared",
  "OnEffectStackDecrease",
  "OnEffectDelayedKnockbackForce",
  "OnEffectCanceled",
  "DebugPrint",
  "DebugAssert",
  "SetProjectileProperty",
  "PreLoadBinks"
}

for _, name in pairs(nop_functions) do
  _G[name] = function(...) end
end


-- Time is not relevant, it's only used to set the fresh file seed and
-- we will overwrite that.
function GetTime(...)
  return 0
end

-- Use english for localization.
function GetLanguage(...)
  return "en"
end

-- For now we don't care about these config options.
function GetConfigOptionValue(args)
  if args.Name == "DebugRNGSeed" then
    return 0
  else
    return nil
  end
end
