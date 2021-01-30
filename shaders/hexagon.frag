#version 450

layout(location = 0) out vec4 outColor;

// warping hexagons, WIP. @psonice_cw
// I'm sure there's a less fugly way of making a hexagonal grid, but hey :)

//  Maybe - Try this...

// Simplify!

// 1 on edges, 0 in middle
float hex(vec2 p) {
  p.x *= 0.57735*2.0;
	p.y += mod(floor(p.x), 2.0)*0.5;
	p = abs((mod(p, 1.0) - 0.5));
	return abs(max(p.x*1.5 + p.y, p.y*2.0) - 1.0);
}

void main(void) { 
	vec2 pos = gl_FragCoord.xy;
	vec2 p = pos/20.0; 
	float  r = (1.0 -0.7)*0.5;	
	outColor = vec4(smoothstep(0.0, r + 0.05, hex(p)));
}