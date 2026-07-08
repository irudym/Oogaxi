## Game design description

### Game objects

* The copter (flying taxi, "Fifth Element style" in the future)
* Passengers
* Platforms
* The food vendor (fuel vendor)
* A pterodactyl (other cars and flying objects in the future)
* Falling rocks (garbage in the future)
* Torches (street lamps in the future)
* Water droplets
* Splash particles


### State Graph
Enter                 crash (K for now)
Menu ───────────▶ InGame ─────────────────────▶ GameOver
▲                  │  ▲                            │
│                  │Esc│Esc     (IsPaused only     │ Enter
└──────────────────┴───┴─────  exists in InGame)  ◀┘
        Running ⇄ Paused
