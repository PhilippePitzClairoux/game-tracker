# Game-Tracker
Personnal tool to help manage my gaming addiction. This program will scan for all
your installed games and track how long you've been playing. Once you've reach that
limit, a notification is sent and all games are killed.

Currently tested on Fedora only - but should technically work on windows as well.
(might need to implement a config file specifying game directories).

### How to build
```shell
cargo build --release
```

### How to run
```shell
game-tracker --hours 2 --minutes 30 # allow 2 hours and 30 minutes of game time
```

## TODO 
- Implement protections to make the program unkillable
- Find other ways to scan for games
- implement tests