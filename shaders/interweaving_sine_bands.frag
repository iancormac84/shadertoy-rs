#version 450

layout(binding = 0) uniform Input {
    vec4 iResolution;
    float iTime;
};

layout(location = 0) out vec4 fragColor;

vec3 calcSine(vec2 uv, 
              float frequency, float amplitude, float shift, float offset,
              vec3 color, float width, float exponent)
{
    float y = sin(iTime * frequency + shift + uv.x) * amplitude + offset;
    float scale = pow(smoothstep(width, 0.0, distance(y, uv.y)), exponent);
    return color * scale;
}

void main(void) {
	vec2 uv = gl_FragCoord.xy / iResolution.xy;
    vec3 color = vec3(0.0);
    
    color += calcSine(uv, 2.0, 0.25, 0.0, 0.5, vec3(0.0, 0.0, 1.0), 0.3, 1.0);
    color += calcSine(uv, 2.6, 0.25, 0.2, 0.5, vec3(0.0, 1.0, 0.0), 0.3, 1.0);
    color += calcSine(uv, 2.9, 0.25, 0.4, 0.5, vec3(1.0, 0.0, 0.0), 0.3, 1.0);
    
	fragColor = vec4(color,1.0);
}