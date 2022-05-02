# Future Work

## Bugs

 * ~~Is the cube decal texture lifetime okay?~~
 * Cube spins too fast at program startup.
 * Anisotropic shading is not aligned to anything.

## Cleanup and Organization

 * ~~remove dead code.~~
 * ~~`FaceInstanceRaw::desc()` should use `ATTRIBUTES` constant.~~
 * ~~rename **cube_face_shader_NEW.**~~
 * ~~rename `cube.texture` to `cube.decal`.~~
 * ~~move camera to its own file.~~
 * ~~need a way to keep track of bind groups.~~
 * move most of the event handling out of **main.rs**?
 * ~~convert trackball to use a quaternion instead of a matrix.~~
 * ~~get rid of the `bindings::Bg` struct.~~
 * ~~add accessor functions to generate `BindingResource`s.~~
 * maybe create a module for shaders and pipelines?
 * ~~move blinky into its own module.~~
 * only push camera and lights uniforms to GPU when they change.
 * ~~use `include_wgsl!` macro instead of `include_str!`.~~

## Features

 * ~~give the trackball momentum.~~
 * set trackball center at cube's center.
 * ~~add lighting.~~
 * ~~add a floor.~~
 * ~~put shadows on the floor.~~
 * implement LED animation.
 * add a mirror.
 * put smudges or patina on the mirror.
 * ~~use multisampling.~~

## ~~Lighting Strategy~~

 * ~~decide how to allocate bind groups.~~
 * ~~define lights and pass them through to the shaders.~~
 * ~~add Burley diffuse lighting to edge shader.~~
 * ~~brute force a shadow map.~~
 * ~~refactor shadow map.~~
 * ~~blur the shadows.~~
 * ~~clip shadow map lookups to in-bounds texels.~~