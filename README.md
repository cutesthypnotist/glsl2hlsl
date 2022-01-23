# glsl2hlsl
Slightly cursed WIP ShaderToy to Unity (ShaderLab) transpiler.

Sorry if the code gives you an aneurism.

# Features
- Converts a fairly decent chunk of shadertoy shaders to usable unity shaders
- Can attempt to automatically find and extract properties from the shader and put then in the inspector
- Can attempt to automatically make raymarched/raytraced shaders 3D
- Can download shaders directly from the shadertoy API given a link
- Synthesizes a usable unity material and .meta file when using download feature

# Some cool shaders to try it on
[Fractal land by Kali](https://www.shadertoy.com/view/XsBXWt)

[Protean clouds by IQ](https://www.shadertoy.com/view/3l23Rh)

[Fractal pyramid by bradjamesgrant](https://www.shadertoy.com/view/tsXBzS)

[Phantom star by kasari39](https://www.shadertoy.com/view/ttKGDt)

# To do
- Better support for preprocessor directives in any context
- Multiline macro support
- Implement various missing sampler-related functions such as textureSize, textureLodOffset etc.
- A few properties defined by shadertoy (iDate, iChannelResolution...) are missing
- Refactor

# Build
`cargo build`

# Usage
`glsl2hlsl <fileToConvert>`

Or just use the website :P

# Issues
- Missing material on buffer CRTs
- In CRT settings -> Material -> Shader pass is reverting to default after re-adding material (i.e.: buffer 1 corresponds to pass "1" in shader, buffer 2 -> pass "2", etc.)
- Materials -> Textures missing from texture slots.
- Cubemaps don't generate links.
- Images don't auto-download.

# Other testing things.
Test shader textures: https://www.shadertoy.com/view/ltScRG
Test shader multi-pass: https://www.shadertoy.com/view/7dyXDz
https://www.shadertoy.com/view/ssGSDh
https://www.shadertoy.com/view/ld3GWS
https://www.shadertoy.com/view/XdVGWt

Common pass:
https://www.shadertoy.com/view/MddcRr

Everything: 
https://www.shadertoy.com/view/wssSD4

Test:
https://www.shadertoy.com/view/flfSzl

