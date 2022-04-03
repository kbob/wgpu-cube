# Notes

Just collecting ideas as I come to understand them...

# Encapsulating Object Models

This page on the wgpu wiki explains a design pattern to encapsulate
graphics work in each object model.

https://github.com/gfx-rs/wgpu/wiki/Encapsulating-Graphics-Work

I tried some things, and wrote a `Renderable` trait that seems to work.
I'm sure it will require refinement.

# Rendering Stages

The Bevy game engine defines six rendering stages.  They are elaborated here.

https://docs.rs/bevy/latest/bevy/render/enum.RenderStage.html

The six phases are
 * Extract
 * Prepare
 * Queue
 * PhaseSort
 * Render
 * Cleanup

 PhaseSort exists because Bevy has a generalized render graph, and it has
 to deduce the order in which to do render passes.  Prepare and Render
 stages are analogous to the Prepare/Render pattern in the wgpu wiki above.

# Categories of GPU Data

Here are three ways to categorize data sent to the GPU.  I'm sure there
are more.

### By Scope
 * uniform (global to render pipeline)
 * per instance
 * per vertex

### By Lifetime
 * static
 * frame
 * render pass
 * pipeline
 * other?

 ### By Visibility
  * vertex
  * fragment
  * both
  * (compute)

# Transformations

(I vaguely remember reading this somewhere 30 years ago.  It's probably
not right. And my copy of Foley and van Dam is lost in the basement.)

There is a stack of transformation matrices.  To map an object into
pixels on the viewport, you multiply its vertex positions by each of these
in turn.

### Spaces
 * **Object** — the object's most natural coordinate system
 * **Parent Object** — same, for the object's parent
 * ...
 * **Root Object** — same, for any object that doesn't have a parent.
 * **World** — the modeled world.  The world would usually be aligned
   up and down and use meaningful units like meters or miles.
 * **View** — the frustum that the camera can see.
 * **Projection** — a unit cube that is the graphics system's (normalized)
   natural coordinates

To render an object, you left-multiply its vertex positions by each
of these transforms in turn.
When it's in Projection space, the GPU can render it into pixels.

Normal vectors are converted into View space, because they're primarily used
for lighting calculations, which are done relative to the camera.

wgpu uses a left handed projection space.  +X is to the right, +Y is up,
and +Z is away from the viewer.  DirectX is the same.  OpenGL and Vulkan
use right handed spaces.  +Z is toward the viewer.

There is a flag in main.rs, `WORLD_HANDEDNESS`, to use either a left-handed or right-handed
world space.  I prefer right-handed, as it's what I'm used to.

I use the convention `<space>_to_<space>` for my transformation matrices.
`face_to_cube`, `cube_to_world`, `world_to_view`, etc.