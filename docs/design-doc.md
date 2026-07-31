# Game Description

*A modern reimagining inspired by the classic Ugh! (DOS, Amiga)*

## Story

The game opens in an alternate prehistoric era. The protagonist is a caveman hopelessly in love with a woman from his tribe, but she won't give him the time of day without gifts and money. Determined to win her heart, he sets up the world's first taxi service: a muscle-powered pedal copter built from sticks and leaves. He earns money by picking up passengers and flying them safely to their destinations — for a fee.

Just as the hero is about to win the woman's heart, he is suddenly pulled into a time portal and thrown into a distant, cyberpunk future. Stranded, he must once again earn money as a flying taxi pilot. This time to buy the parts for a time machine, assemble it, and find his way back home to his beloved.

## Structure

The game is divided into two distinct settings:

1. **The Alternate Prehistoric Era** — a stone-age world of dinosaurs, pterodactyls, and primitive currency, where the player pilots the pedal copter.
2. **The Cyberpunk Future** — a neon-lit metropolis where the player controls a modern flying taxi with rocket engines.

## Core Gameplay

- **Damage system:** Collisions with obstacles, hard landings, and rotor or engines contact with the environment all damage the helicopter.
- **Stamina system:** 
* Prehistoric:
  - Pedaling the copter gradually exhausts the pilot. Stamina can be restored by eating fruit, which is knocked out of trees by dropping a stone on them, or buying the food at food shop with money.  
* Future:
  - driving the taxi consumes the fuel, which needs to be refill at gas stations. 
- **Combat:** The stone can also be dropped on hostile monsters to knock them out.
- **Economy:** Fares earned from passengers can be spent in the shop on food (or fuel, in the future segment).

## Obstacles

**Prehistoric era:**
- Dinosaurs
- Flying pterodactyls
- Floods
- Falling rocks
- Monkeys swinging on lianas

**Cyberpunk future:**
- Robots
- Flying cars
- Falling debris (garbage, bricks)
- Criminals and sky pirates  

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

### Input pipeline
devices → [leafwing] → ActionState (verbs) → accumulate_input → ThrustInput (intent) → FixedUpdate sim

### Click execution
gathering (leafwing, render clock) → intent (ours, plain data) → simulation (fixed clock)


### Passenger behaviour

Emerging (steps out of cave portal)
  → WalkingToSign            (walks to their stop's sign)
  → Waiting                  (idles at the sign)
  → Announcing               (the copter is nearby → speech bubble shows destination address)
  → Boarding                 (copter lands at their stop → walks to the copter)
  → Riding                   (hidden aboard; fare meter live)
  → Unboarding               (copter lands at ANY stop (expect the passenger's origin) → walks out onto the pad)
  → Leaving                  (walks to the nearest cave entrance)
  → despawned at the portal  (the Remove observer eulogizes)

Wrong address at unboarding: rude bubble → Leaving, fare unpaid.
The passenger can unboard the copter only if the address on arrival is not equal to the origin address of the passenger.

### Hazards
**Pterodactyl** has a patrolling behaviour be default, when spot the player in the vicinity, shows a speech bubble with '!' and start attacking the player (dive attack).

### Effects

* **Screen Shake** - Trauma-based screen shake (Squirrel Eiserloh’s model — the reference) on severe impact.
* **Dust** - Landing kicks up dust; a wall-scrape throws dust and sparks (in future segment);


### Lighting System

THe lighting system consists of three pieces:
- Offscreen texture
- A camera, that renders only lights
- A material, that multiplies it over the scene  


### Proposals

#### Honking
The honk’s third job. Honking scares creatures: Honked within radius → interrupt to Recovering (flee-flavored). Designer’s call whether this ships — it makes the honk defensive, which changes its economy — but wire it behind a dial and playtest before voting.

#### Night fares
Night fares paying more is what turns a lighting feature into a gameplay one — suddenly the dark is both a hazard and an opportunity, and the player has a reason to fly during the scary part of the cycle. That's a one-line change in fare_between (multiply by a factor derived from DayTime.0.fraction()) plus a HUD hint so the player can see the premium.
