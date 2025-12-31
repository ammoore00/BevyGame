# Level Architecture
Copy/paste:

│   

├── 

└── 

## Player-Facing
```
World
├── Story
│   ├── Level Maps
│   │   ├── Semi-Random Layouts
│   │   └── Level Map Setpieces
│   └── Story Boss
├── Level Bosses
│   ├── Semi-Random Layouts
│   ├── Boss
│   ├── Setpieces
│   └── Injectable Events
└── Side Dungeons
    ├── Semi-Random Layouts
    ├── Setpieces
    └── Injectable Events
```

## Backend
### Overview
```
World
├── Palettes
│   ├── Level Bosses
│   │   ├── Level Bosses
│   │   └── Setpieces
│   ├── Side Dungeons
│   │   └── Setpieces
│   ├── Injectable Events
│   └── Level Map Setpieces 
└── Story
    ├── Level Maps
    └── Story Boss
```

### Maps
```
Map
├── Definition
│   ├── Palette
│   └── Layout
│       ├── Setpieces/Bosses
│       ├── Connectors
│       │   ├── Bounds
│       │   └── Connections
│       │       └── Connection Type
│       ├── Injectable Events
│       │   ├── Bounds
│       │   └── Connector
│       │       └── Connection Type
│       └── Map Transitions
│           ├── Desired Map Type
│           └── Valid Palettes 
├── Storage
│   ├── Setpiece/Boss States
│   └── Enemy Alive/Dead
└── In-Memory
    ├── Chosen Map
    │   └── Chosen Injectable Event(s)
    ├── Connectors
    ├── Baked Tile Grid
    └── Enemy State
```

### Rooms
```
Room
├── Type
├── Bounds
├── Connections
│   └── Connection Type
├── Enemies
├── Items
└── Room Data
```