# routefinder
Standalone routing tool for Hades

Loads the game files, mods, and a save file, and can run them with game-accurate RNG. Currently hard-coded to expect to run Ello's starting boon selector
```
$ cargo run -- /Users/Shared/Epic\ Games/Hades/Game.macOS.app/Contents/Resources/Content/Scripts --save_file ~/Library/Application\ Support/Supergiant\ Games/Hades/Profile4.sav
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
