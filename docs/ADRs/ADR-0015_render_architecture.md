# ADR-0015 — Layered Render Architecture: Scene Texture, Tracking Cameras, and the Composite Stack

**Status:** Accepted
**Date:** 2026-08
**Supersedes:** the ad-hoc overlay-quad approach recorded in ADR-0013 (post-process) and ADR-0014 (lighting composite)

---

## Context

Three separate effects independently demanded the same capability — read the rendered scene as a texture:

- **Refraction**: water must sample and distort what's behind it.
- **Desaturation / colour grading**: pulling a pixel toward its own luminance requires knowing that pixel.
- **Palette-ramp lighting**: mapping a lit pixel back onto an authored OOGA-64 ramp requires reading the unlit pixel first.

Until now these were approximated with full-screen quads on an overlay camera, compositing *after* tonemapping via blend-state tricks. That approach has hard limits: a fragment shader cannot read the destination pixel, so anything requiring the scene as *input* (rather than as a blend operand) was impossible. Three deferred effects wanting the same missing piece is the signal that the escalation is worth paying for.

A second, independent problem surfaced while implementing water: effects that sample the scene are drawn from **world-space** geometry (a pond sits at a level position), but the scene texture they sample is **screen-space** (it already contains the camera's view). Mixing the two on one camera caused double-scrolling — the water moved once because the camera moved, and again because the texture content moved.

## Decision

**1. The game camera renders into a `SceneTexture` rather than the window.** A stationary present quad blits it back, giving every downstream effect the scene as a sampleable input.

**2. Every effect that samples the scene gets its own tracking camera, its own render texture, and a present quad in the composite stack.** Not per-entity screen-space position maths.

**3. Cameras are classified by the space their contents live in:**

> **A camera tracks the game camera if and only if its contents are world-space.**
>
> - Lights sit at torch positions → world-space → tracks.
> - Water sits at pond positions → world-space → tracks.
> - The scene quad, light composite, refraction composite, and vignette are full-screen images already containing the view → screen-space → stationary.

This single test determines which camera any future effect belongs to.

**4. The composite stack is ordered by physics, not convenience.** Each present quad is inserted at the depth its effect actually occupies: water composites *before* lighting, so lighting affects water; the vignette composites last, so it darkens the finished image.

### The resulting pipeline

| order | camera | tracks | layer | target | contents |
|---|---|---|---|---|---|
| −1 | light | yes | 2 | LightMap texture | light fan meshes |
| 0 | game | — | 0 | Scene texture | the world |
| 1 | refraction | yes | 3 | Refraction texture | water meshes |
| 2 | overlay | no | 4 | **window** | the composite stack + UI |

The overlay camera draws four screen-space quads, in z order:

| z | material | blend | purpose |
|---|---|---|---|
| 0 | `ScenePresentMaterial` | opaque | the world |
| 1 | `RefractionPresentMaterial` | **alpha blend** | water over the world |
| 2 | `LightCompositeMaterial` | **multiply** | darkens world *and* water |
| 3 | `ScreenMaterial` | alpha blend | vignette, scanlines |

### Load-bearing details

- **The refraction camera clears to `Color::NONE`.** Its texture is mostly empty; an opaque clear would black out the scene beneath. Its present quad must be `AlphaMode2d::Blend`, not `Opaque`.
- **Only the first window-drawing camera clears.** Everything above it uses `ClearColorConfig::None`.
- **All cameras share one projection** (`ScalingMode::Fixed`, `VIRTUAL_RES`) from a single constant. A 640×480-vs-640×360 mismatch between the scene texture and the overlay projection previously caused stretched output; one constant makes it impossible.
- **UI targets the topmost window camera** via `IsDefaultUiCamera`, so it draws over lighting and vignette rather than being dimmed by them.
- **One generic `sync_tracking_cameras` system** serves all tracking cameras via a `TracksGameCamera` marker, replacing per-camera copies.

## Consequences

**Gained:** refraction, desaturation, and palette-ramp lighting all become possible; lighting affects water correctly rather than water floating unlit above a lit scene; a general recipe for future scene-sampling effects (heat haze, glass, portals) with no new architecture required.

**Costs:** two additional full-screen textures (~1.8 MB at 640×360 RGBA8); two extra composite passes; five cameras to keep straight.

**Accepted risks and their mitigations:**

- **Camera multiplicity is the project's most frequent bug source** — four separate incidents (`camera_follow` silently skipping on ambiguity, egui attaching to the light camera, the overlay projection mismatch, water on the wrong layer). Mitigated by: marker components on every camera, zero bare `With<Camera2d>` queries, and a dev `audit_cameras` system logging order/layer/target for every camera at startup.
- **HDR is capped by the texture format.** `Rgba8Unorm` clamps at 1.0, so bloom (which runs inside the game camera's pipeline, before the texture write) still works, but further HDR work downstream is limited. `Rgba16Float` is the fix at double the memory, if needed.
- **Water samples the pre-lighting scene**, so refracted content shows unlit colours before the final multiply. Invisible in practice — the sampled pixels sit within a few pixels of the water and receive near-identical light — and not worth correcting.

## Alternatives considered

**Keep the blend-trick approximations.** Rejected: three effects were already blocked, and each approximation was measurably worse than the real version (the alpha-blend light composite produced fog rather than lighting; multiply-only water could not desaturate or refract).

**Per-entity screen-space position maths** instead of tracking cameras. Rejected: it does not scale — every pond, waterfall, and future effect entity would need its own sync system, whereas a camera performs the transform once for all of them.

**A custom render-graph node.** The "proper" solution, and genuinely more capable (it could composite before tonemapping). Rejected for now: the render graph is Bevy's steepest and most churn-prone API, and it is exactly the layer where the 2D lighting crate ecosystem went stale (ADR-0014). The camera-and-texture approach achieves the same results using stable, documented APIs.

**Reversal trigger:** if an effect requires compositing *before* tonemapping — true HDR-space distortion, or physically-correct heat haze — the render-graph node becomes necessary, and this architecture should be revisited as a whole rather than extended further.
