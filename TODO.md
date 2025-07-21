1. version of SetupFindIncrementally that cleanly resumes from a previous state
2. dynamic route length - when looking of Impending, stop when we find it instead of always continuing to C17
3. introduce certain (worse) searches with a penalty ie. start searching for epic special, but search for rare with 8 increment lag
4. enemy scoring and filtering out routes with too high total enemy difficulty (post-scoring?)
5. improve (declarative?) requirements format
6. optimization - pre-filter GameState, CurrrentRun
7. optimization - pre-filter Rooms (ie. in RoomData prior to running)
8. tool - save "room snapshots" in some (json) format (for later inspection)
9. tool - convert save files to same format (for later comparison)