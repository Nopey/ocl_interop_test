# rust + ocl + sdl2 + gl
I tried to make them work together.
<<<<<<< HEAD
(they don't)
maybe later
=======
(they didn't)
>>>>>>> be23d2947577f66a6ee3c0c9e8e2145049a0864d

I'm unsure why, but SDL_GLRender is a \*const c_void, and the gl_context function takes a u32.

For now it's just a pipe dream
