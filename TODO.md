# Future Work

## Bugs

 * Is the cube decal texture lifetime okay?
 * Cube spins too fast at program startup.

## Cleanup and Organization

 * ~~remove dead code.~~
 * ~~FaceInstanceRaw::desc() should use ATTRIBUTES constant.~~
 * ~~rename cube_face_shader_NEW.~~
 * ~~rename cube.texture to cube.decal.~~
 * ~~move camera to its own file.~~
 * need a way to keep track of bind groups.
 * move most of the event handling out of main.rs?
 * convert trackball to use a quaternion instead of a matrix.

## Features

 * give the trackball momentum.
 * set trackball center at cube's center.
 * add lighting.
 * implement LED animation.
 * add a floor.
 * put shadows on the floor.
 * add a mirror.
 * put smudges or patina on the mirror.
 * use multisampling.
