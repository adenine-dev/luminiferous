# Luminiferous

This is a hobby ray tracer. Very much a work in progress right now. Commits are going to be a mess for secret reasons :).

## Renders

(These are each 4k so they may take a bit to load)

![Monkey](./renders/monkey.png)

<p align="middle">Suzanne, 1024 samples</p>
<p align="middle">This is a basic render that demonstrates various capabilities of the renderer including ability to load models, textures, and analytic materials.</p>

![Bunny](./renders/bunny.png)

<p align="middle">Bnuuy, 1024 samples</p>
<p align="middle">This demonstrates the media rendering capabilities of the renderer. The bunny has a dielectric surface with a homogeneous internal participating medium which results in a jade or soap like surface.</p>

![Vintage Oil Lamps](./renders/vintage-oil-lamps.png)

<p align="middle">Vintage Oil Lamps, 1024 samples</p>
<p align="middle">This render uses the measured material rendering capabilities of the renderer. Each material in this scene (barring the glass chimneys) is sourced from <a href="https://rgl.epfl.ch/materials">the RGL material database</a>.</p>

![Misty Dragon](./renders/misty-dragon.png)

<p align="middle">Misty Dragon, 2048 samples</p>
<p align="middle">This uses the volumetric rendering capabilities and a spotlight to render a model dragon in a scene wide mist.</p>
