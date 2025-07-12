# routefinder
Standalone routing tool for Hades

Loads the game files, mods, and a save file, and can run a lua script in that environment with game-accurate RNG.

```
$ cargo run -- run -f ~/Library/Application\ Support/Supergiant\ Games/Hades/Profile4.sav ./StartingBoonDemo.lua
SecondRoomReward        LockKeyDropRunProgress
SecondRoomChaos false
Type    Hammer
FirstRoomChaos  false
FirstRoomShrine false
SecondRoomName  A_Combat06
HammerData
  Options
    1
      Name      GunSlowGrenade
    2
      Name      GunHomingBulletTrait
    3
      Name      GunExplodingSecondaryTrait
SecondRoomRewardStore   MetaProgress
...
11
  Waves 1
  Seed  906036749
  Enemies
    PunchingBagUnit     true
  Exits
    1
      Reward    RoomRewardMaxHealthDrop
      ChaosGate false
      Room      A_Combat08A
    2
      Reward    RoomRewardMoneyDrop
      ChaosGate true
      Room      A_Combat14
  Uses  20
...
```

set `HADES_SCRIPTS_DIR` to the Scripts directory of your hades install to avoid needing to pass it in every time
