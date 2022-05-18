# wgpu Recipes

This is a set of checklists for adding functionality to
wgpu-based projects.  It assumes that bind groups are
managed in src/binding.rs

* `Foo` is an object (not necessarily geometric)
* `Bar` is a texture
* `Fred` is a bind group
* `Barney` is a binding in a bind group

## Add an object

 * create src/foo.rs
 * add "`mod foo;`" to src/main.rs
 * define `struct Foo`
 * define vertex input
 * (optional) define instance input
 * write `Foo::new`
   - create and initialize uniform buffer
   - create and initialize vertex buffer
   - (optional) create and initialize instance buffer
   - create and initialize any textures
 * add uniform and textures to bind groups
 * implement `Renderable` for `Foo`
 * (optional) implement `Foo::update`
 * (optional) implement `Foo::resize`
 * instantiate `Foo` in `main::State::new`
 * call `self.foo.Prepare` and `Foo::Render` from `main::State::render`
 * (optional) call `self.foo.update` from `main::State::update`
 * (optional) call `self.foo.resize` from `main::State::resize`

## Add a uniform object

* define `struct FooUniformRaw`
* declare `Foo::uniform_buffer`
* in `Foo::new()`, create and optionally initialize `uniform_buffer`
* in `Foo::update()`, optionally update `uniform_buffer`
* add to bind group (see below)
* define `FooUniform` (the struct definition) in shaders
* declare `foo` (the instance) in shaders
* write code to use `foo`

## Add a vertex input

* define `struct FooVertexRaw`
* define `FooVertexRaw::ATTRIBUTES`
* write `FooVertexRaw::desc<'a>()`
* write `to_raw()`
* add `Foo::FooVertexRaw::desc()` to RenderPipelineDescriptor
* define `FooVertexInput` in vertex shader
* write vertex shader

## Add a field to a vertex input

* Add field to `FooVertexRaw`
* Add field to `FooVertexRaw::ATTRIBUTES`
* Initialize field in `FooVertex::new()` or other constructor
* Add code to `FooVertex::to_raw()`
* add field to `FooVertexInput` in vertex shader
* write code in vertex shader

## Add an instance input

Pretty much the same as adding a vertex input

## Add a field to an instance input

Pretty much the same as adding a field to a vertex input.

## Add a uniform binding to a bind group

* define `MyBindings::FOO_UNIFORM as the next available number
* add a `BindGroupLayoutEntry` to `MyBindings::new`
* add a `BindingResource` argument to `MyBindings::create_bind_group`
* add a `BindGroupEntry` to `create_bind_group`
* add an argument to `create_bind_group`'s callers
* define `FooUniform` in shader(s)
* declare `foo`, the `FooUniform` instance, in shader(s)
* write code in shader(s)

## Add a texture and sampler

* add `bar_view` and `bar_sampler` to `struct Foo`
* create texture data
* create `bar_texture`, `bar_view`, and `bar_sampler` in `Foo::new`
* add bindings for view and sampler to appropriate bind groups
* declare `t_bar` and `s_bar` in fragment shader
* write code in fragment shader

## Add a dynamic texture

In addition to the steps to add an object and a static texture (above),

* add `bar_texture` to `struct Foo`
* implement `Renderable`
* in `render`, write texture contents

## Add a bind group

* define `FredBinding` in src/binding.rs
* define `FredBinding::GROUP_INDEX`
* define constants for all bindings in group
* create `BindGroupLayout` in `FredBinding::new`
* write `FredBinding::create_bind_group`
* create `FredBinding` in appropriate object(s)
* create bind group in appropriate object(s)
* set bind group in objects' `render` methods
* declare types and variables in shaders
* write code in shaders

## Add a binding to a bind group

* define `FredBinding::BARNEY_UNIFORM` (or `_TEXTURE` or `_SAMPLER`)
* add BindGroupLayoutEntry to `FredBinding::new`
* add parameter and BindGroupEntry to `FredBinding::create_bind_group`
* add parameter to calls to `FredBinding::create_bind_group`
* add type and variable to shaders
* write code in shaders

## Add a render pass

TBD

## Add a render pipeline

TBD

## Render an object

## Use a dynamic offset

TBD
