# routefinder
Standalone routing tool for Hades

Load's the game files + mods, and can run them with game-accurate RNG. Currently hard-coded to expect to run Ello's starting boon selector
```
$ cargo run --  /Users/Shared/Epic\ Games/Hades/Game.macOS.app/Contents/Resources/Content/Scripts -s 19986 -w GunWeapon -i 2
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
```
