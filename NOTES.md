# Notes

Just collecting ideas as I come to understand them...

# Encapsulating Object Models

This page on the wgpu wiki explains a design pattern to encapsulate
graphics work in each object model.

https://github.com/gfx-rs/wgpu/wiki/Encapsulating-Graphics-Work

I tried some things, and wrote a `Renderable` trait that seems to work.
I'm sure it will require refinement.

# Rendering Stages

The Bevy game engine defines six rendering stages.  They are elaborated
here.

https://docs.rs/bevy/latest/bevy/render/enum.RenderStage.html

The six phases are
 * Extract
 * Prepare
 * Queue
 * PhaseSort
 * Render
 * Cleanup

PhaseSort exists because Bevy has a generalized render graph, and it
has to deduce the order in which to do render passes.  Prepare and
Render stages are analogous to the Prepare/Render pattern in the wgpu
wiki above.

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
pixels on the viewport, you left-multiply its vertex positions by each
of these in turn.

### Spaces
 * **Object** — the object's most natural coordinate system
 * **Parent Object** — same, for the object's parent
 * ...
 * **Root Object** — same, for any object that doesn't have a parent.
 * **World** — the modeled world.  The world would usually be aligned
   up and down and use meaningful units like meters or miles.
 * **View** — the frustum that the camera can see.
 * **Projection** — a unit cube that is the graphics system's
   (normalized) natural coordinates

To render an object, you left-multiply its vertex positions by each
of these transforms in turn.  When it's in Projection space, the GPU can
render it into pixels.

Normal vectors are converted into View space, because they're primarily
used for lighting calculations, which are done relative to the camera.

wgpu uses a left handed projection space.  +X is to the right, +Y is up,
and +Z is away from the viewer.  DirectX is the same.  OpenGL and Vulkan
use right handed spaces.  +Z is toward the viewer.

There is a flag in main.rs, `WORLD_HANDEDNESS`, to use either a
left-handed or right-handed world space.  I prefer right-handed, as it's
what I'm used to.

I use the convention `<space>_to_<space>` for my transformation
matrices.  `face_to_cube`, `cube_to_world`, `world_to_view`, etc.

# Bind Groups and Multipass Rendering

Thinking out loud...

If I'm going to do shadows and reflections, I need to have a multipass
rendering strategy.  Different passes have different data needs.

There are only four bind groups by default.  The bind group assignment
should be closely affiliated with the shader, as they're declared in
the shader.

Is there any limitation on the number or kinds of bindings in a group?
Could I just jam everything into a single bind group?

So the object model (entity?) is kind of independent of the shader and
render pass.  Shadow passes just need accurate vertex information.
Textures and samplers are associated with models, though.

Instead of statically allocating bind groups and letting all the shaders
share them,

Maybe, if I write one of those generic shaders, I can get by with two
shaders: one for the LED surfaces and one for everything else.

The first bind group should hold anything that's truly static --
textures, xforms for static objects, light properties?

The second bind group should be things that are used by several/all
shaders.

The third should hold things that can change per render pass?

### *SpongeBob narrator voice: one week later...*

The shader knows all about the bindings.  They are declared in the
shader source.  Group, position, type, and fields.  Entities know
about the individual variables.  Type, fields, initial values, and
when to update.

In fact, the entity does not know about the shader.  It will be rendered
by different shaders at various times.  There should be a level of
indirection between the entities and the bind groups: the entity updates
the variables' values, and the shaders reference the bindings.

    impl ShaderVariable {
      fn rebind<'a>(&mut self, resource: wgpu::BindingResource<'a>);
    }

Then the level of indirection will map between variable *names* and
the ShaderVariable object.  *Names* in italics because a name could be
a simple string, an ID number, or something more structured.

The map is pretty static.  It should be a ref to the ShaderVariable?

Then when bind groups are created, they get the current resource values
from the variables.

So the Binding is associated with the shader and pipeline, and it
knows the name and the binding group/position.  The ShaderVariable is
independent.  It knows the name, type, and fields.  The Entity
knows when to update the variables' values.

I *could* make the Binding generate shader source code for the bindings.
Then I'd have to have some sort of DSL to declare the bindings in Rust
source, compile to WGSL source, and prepend it to the shader source.

A Binding has a layout and can create a BindGroup with the current
resources.

    impl Binding {
      pub layout: wgpu::BindGroupLayout;
      fn new(vars: &[ShaderVariableName]) -> Self;
      fn create_bind_group(device) -> wgpu::BindGroup;
    }

    impl ShaderVariable {
      fn new(name, type, fields) -> Self;
      fn get(&self) -> resource;
      fn set(&mut self, new_resource);
    }

    type ShaderVariableDirectory = std::HashMap<String, ShaderVariable>;
    
# References

## Quaternions and Rotations

### **[Exponentially Better Rotations](https://thenumbat.github.io/Exponential-Rotations/)**
*Max Slater*<br>
Rotation math.  Hurts head.

### **[Visualizing Quaternions](https://eater.net/quaternions)**
*Grant Sanderson, Ben Eater*<br>
Rotation math.  Hurts head but with "interactive video".

## Deferred Rendering

### **[Deferred Shading](https://learnopengl.com/Advanced-Lighting/Deferred-Shading)**
*Learn OpenGl*
Explains deferred shading and implements it in OpenGL.

### **Deferred Shading**
*Etay Meiri, OGL dev*<br>
Another deferred shading tutorial in OpenGL.<br>
[Part 1](https://ogldev.org/www/tutorial35/tutorial35.html)<br>
[Part 2](https://ogldev.org/www/tutorial36/tutorial36.html)<br>
[Part 3](https://ogldev.org/www/tutorial37/tutorial37.html)<br>

## Shadow Maps

### **[Real-time Shadows](https://docs.google.com/presentation/d/1MwJcnSvkAzpT8BuoSqIkzlYLjdA_lBDrt8bW-vcwmDU/edit#slide=id.p)**
*Javi Agenjo 2020*<br>
Slide show gives a good overview of shadow mapping techniques.

### **[Depth Precision Visualized](https://developer.nvidia.com/content/depth-precision-visualized)**
*Nathan Reed, NVIDIA Developer Content*<br>
Explains depth buffer resolution nonlinearities.
Basically, Znear = +1, Zfar = 0 is best.  Zfar = +1 (DirectX) or -1
(OpenGL) significantly reduces a floating point Z buffer's linearity.

### **[Shadow Map Antialiasing](https://developer.nvidia.com/gpugems/gpugems/part-ii-lighting-and-shadows/chapter-11-shadow-map-antialiasing)**
*Michael Bunnel, Fabio Pellacini, GPU Gems*<br>
How to multisample a shadow map for smooth edges

## Virtual Trackballs

### **Virtual Trackballs Revisited**
*Henriksen, Sporing, Hornbaek*<br>
DOI:10.1109/TVCG.2004.1260772<br>
Reviews and compares three trackball models.

There's another paper on 
### **[Virtual Trackballs and the Exponential Map](http://math.umd.edu/~gogo/Papers/trackballExp.pdf)**
*Stantchev, U. of Maryland*<br>
This appears to be a good trackball model.

## Disney Shading

### **[Physically Based Shading at Disney](https://media.disneyanimation.com/uploads/production/publication_asset/48/asset/s2012_pbs_disney_brdf_notes_v3.pdf)**
*Brent Burley, Walt Disney Animation Studios*<br>
Explains Disney Animation's PBR material model in 2012.<br>
[SIGGRAPH 2012 Course](https://web.archive.org/web/20170531155921/http://blog.selfshadow.com/publications/s2012-shading-course/)<br>
[SIGGRAPH 2012 Slides](https://web.archive.org/web/20170531155921/http://blog.selfshadow.com/publications/s2012-shading-course/burley/s2012_pbs_disney_brdf_slides_v2.pdf)<br>
[Disney BRDF Explorer](https://github.com/wdas/brdf)<br>

### **[Implementing Disney Principled BRDF in Arnold](https://web.archive.org/web/20170602195106/http://shihchin.com/2015/07/implementing-disney-principled-brdf-in.html)**
*Shih-Chin, 2015*<br>
[Source Repository](https://github.com/shihchinw/rlShaders)<br>
[Dev Reel](https://vimeo.com/150344036)<br>

## Advanced Lighting

### **[Advanced Lighting: Bloom](https://learnopengl.com/Advanced-Lighting/Bloom)**
*Learn OpenGL*<br>

## Tools

### **[Debugging with Xcode](https://github.com/gfx-rs/wgpu/wiki/Debugging-with-Xcode)**
*Joshua Groves, wgpu-rs Wiki*<br>
You can run wgpu programs under Xcode to use the Metal inspector.

### **[Developing and Debugging Metal Shaders](https://developer.apple.com/documentation/metal/developing_and_debugging_metal_shaders)**
*Apple*
Xcode has a very useful Metal debugging inspector.
