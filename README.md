# snek?
Welcome to snek?, my Acerola Game Jam 0 entry!

## 🍎 Gameplay Overview
The game starts as a normal game of the classic arcade game Snake, but new mechanics are quickly introduced. The game's objective is to eat all of the fruit.

## 🛠 Technologies & Credits 
- Code: Rust 2021 🦀
- Graphics: OpenGL 4.5 + GLFW
- Audio: Soloud
- Sounds: me, with sfxr
- Icons: me, with paint.net

## 🧠 Post-Mortem 
I'm new to game development & graphics programming, so I wanted to push my limits by writing a game from scratch, keeping my dependency on third-party abstractions to a (sane) minimum.\
In retrospect, writing an engine is a serious undertaking!

### 👶 0. The beginning 
A few weeks ago I wrote my first OpenGL app, a [basic 3D renderer](https://github.com/truenotzero/kef), written in pure C, supporting a (single) dynamic light and shadowmaps, but that's it. This marked the beginning of my journey into game development. I also wrote a [small app with Rust + OpenGL](https://github.com/truenotzero/deer-defense) for a challenge by YouTuber LowLevelGameDev, however here the focus was on the multiplayer side of things more than anything else. For this game jam (technically my first, since my last app was submitted to a challenge and not a game jam), I wanted to break out of my comfort zone: I wanted to explore different rendering techniques while focusing on graphical effects.

### 🧩 1. Writing a 2D Tile Renderer: It's like 3D, but with its own challenges
As my main focus is expanding my OpenGL skillset, I wanted to implement instanced-based rendering.
I figured that a 2D tile-based renderer would be a perfect use case. This is the meat of the rendering engine, where each tile can be colored and transformed independently.

With the tile renderer implemented, I set my sights on implementing some 'eye candy'. First, I implemented animations for the player (which prompted me to rewrite my renderer 😂). Next, I wanted to implement a fireball-like projectile. This posed a few challenges along the way:
- My current renderer only supports rendering quads, so I wrote a new renderer.
- My current entity system only knows how to work with the tile renderer, so I rewrote the entity system to support both renderers.
- The approach when coding shaders is nothing like when writing 'normal' programs... How to achieve the graphical effects I want? My solution was to boot up Shadertoy and play around, learning the effects various mathematical functions have on the shapes I can draw, thus building a 'feel' for how to do things.

Next, I wanted to implement a forcefield:
- My current renderers (tile & fireball) can't achieve the graphical effect I am looking for, so I wrote yet another renderer.
- My initial approach was to generate a forcefield in a given quad, and then 'remove' some of the sides to make it look contiguous.
- I couldn't figure this out at first! I spent over a day on the shader, rewrote it from scratch about 4 times but I enjoyed writing this one the most.
- Evenntually, I tried a completely different approach. The approach that I ended up taking is that the shader 'builds' the forcefield around the player additively, by being told in which directions to do so.
- By this point I already had 3 different pipelines, so I didn't want to just 'slap' another renderer on my entity system. I wrote an abstraction around my pipelines, making my renderer easier to use.

### 🐍 2. Designing an Entity System: Can't have it all
Designing an entity system from the ground up is a big challenge! I wound up rewriting it three times. 
My considerations, in descending order of importance, leading to my design were:
- Breaking up entities and their behaviors, effectively making behaviors into reusable bits
- Ergonomics
- Fast iteration times & ease of implementing new entities/behaviors

A massive oversight in my original design was collision detecion. I don't like my current implementation, and my next game will likely focus on how to improve entity systems in general and collision detecion specifically.

### 🌎 3. Building a World: Procedural or Deterministic? Why not both!
