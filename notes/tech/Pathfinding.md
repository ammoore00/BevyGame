In a Bevy / ECS context, your current structure (a single monolithic `Pathfinder` component containing a nested state enum with `Path`, `TilePath`, and variants like `Wander` vs `Target`) works for basic cases, but **it runs counter to ECS best practices as capabilities expand**.

Here is an analysis of the structural limitations in your current design and a more scalable, idiomatic Bevy ECS architecture for pathfinding and navigation.

---

### Key Issues in the Current Architecture

1. **ECS Data Coupling & Monolithic Enums**
    * Storing `Path` variants like `Target { target: Entity, ... }` inside an enum field in a single `Pathfinder` component forces systems to handle all path types together.
    * `Path::Wander` and `Path::Target` have different requirements (e.g., target tracking, re-pathing logic, lost target handling). Nesting these inside `PathfinderState` means systems like `wander.rs` or `follow.rs` need complex nested matching and internal state resets.

2. **Mixing Decision (AI), Request (Pathfinding), and Execution (Movement)**
    * Currently, `PathfinderState::Searching` is used as an intermediate step to signal "needs path computation," while `PathfinderState::Moving` holds the active waypoint path, and `AiState` drives when to update wander/follow state.
    * Coupling path calculation requests directly into the component state makes async/deferred pathfinding or multi-frame path requests difficult to manage.

3. **Entity Lifecycle / Ref-Tracking in Enums**
    * Storing `Entity` references inside enum variants (`Path::Target { target: Entity }`) can lead to dangling entity references if the target is despawned without triggering an update to the pathfinder state.

---

### A Better Bevy ECS Representation

Instead of holding states inside nested enums on one component, **decouple pathfinding into distinct components and requests**.

#### 1. Decouple "Intent/Goal", "Active Path", and "Movement Execution"

Represent different parts of the pathfinding pipeline as separate components or transient components/events:

* **`NavigationTarget` (or `FollowTarget`) Component**:
  Attaches to entities that want to track/follow something or move to a goal.
```rust
#[derive(Component)]
  pub struct FollowTarget {
      pub target: Entity,
      pub stop_distance: f32,
      pub last_known_pos: Vec3,
      pub re_path_timer: Timer,
  }
```


* **`PathfindRequest` Component (or Trigger/Event)**:
  When an entity (whether wandering, chasing, or patrolling) decides it needs a new path, add a `PathfindRequest` component to it.
```rust
#[derive(Component)]
  pub struct PathfindRequest {
      pub start: Vec3,
      pub destination: Vec3,
      // Optional settings like clearance, speed, etc.
  }
```

A dedicated `pathfinding_system` queries `(Entity, &PathfindRequest)`, computes `find_path`, attaches a `Waypoints` component, and removes `PathfindRequest`.

* **`Waypoints` Component (Active Path)**:
  Holds the current path navigation state.
```rust
#[derive(Component, Debug)]
  pub struct Waypoints {
      pub points: Vec<Vec3>,
      pub current_index: usize,
      pub target_destination: Vec3,
  }
  
  impl Waypoints {
      pub fn current(&self) -> Option<Vec3> {
          self.points.get(self.current_index).copied()
      }
      pub fn is_reached(&self) -> bool {
          self.current_index >= self.points.len()
      }
  }
```


* **Movement Controller**:
  System simply checks for `&Waypoints` on entities with `&mut MovementController`. If present, it calculates movement intent towards `waypoints.current()`. Once reached, it increments `current_index` or removes `Waypoints` when finished.

---

### Benefits of the Decoupled Approach

1. **Separation of Concerns**:
    * **Wander / Chasing systems** only decide *where* to go and attach/update destination goals.
    * **Pathfinder system** only handles graph traversal / Theta* math when given a request.
    * **Movement system** only follows `Waypoints` into physical movement input.
2. **Reusability**: Any entity (NPCs, enemies, companions, pets) can use `Waypoints` and `PathfindRequest` without needing full `AiState` or `RandomWander` setups.
3. **Bevy Query Performance**: Filtering with query terms like `With<PathfindRequest>` or `With<Waypoints>` is faster and cleaner than branching on nested `match pathfinder.state` inside loops.
4. **Clean Observer/Event Integration**: Target gain/loss observers (`On<GainedTarget>`, `On<LostTarget>`) can directly insert or remove `FollowTarget` or `Waypoints` components rather than reaching deep into enum fields.