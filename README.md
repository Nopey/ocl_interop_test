# rust + ocl + sdl2 + gl
I tried to make them work together.
(they didn't)

I'm unsure why, but SDL_GLRender is a \*const c_void, and the gl_context function takes a u32.

For now it's just a pipe dream
