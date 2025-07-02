#version 330 core

layout(location = 0) in vec3 aPos;
layout(location = 1) in vec3 aNormal;
layout(location = 2) in vec2 aTexCoord;

out vec2 vTexCoord;
out float vFogFactor;
out vec3 vNormal;
out vec3 vWorldPos;
out mat3 vTBN;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

uniform float fogDensity = 0.004;
uniform float fogGradient = 1.5;
uniform float fogStart = 100.0;

void main() {
    vTexCoord = aTexCoord;
    vNormal = mat3(transpose(inverse(model))) * aNormal;
    
    vec4 worldPosition = model * vec4(aPos, 1.0);
    vWorldPos = worldPosition.xyz;
    
    // Calculate TBN matrix (tangent, bitangent, normal)
    vec3 N = normalize(mat3(model) * aNormal);
    vec3 T = vec3(0.0);
    vec3 B = vec3(0.0);
    
    // Generate tangent and bitangent if we have texture coordinates
    if (length(aTexCoord) > 0.0) {
        // Create a temporary normal in case N is zero
        vec3 tempNormal = (length(N) > 0.0) ? N : vec3(0.0, 0.0, 1.0);
        
        // Calculate tangent
        vec3 c1 = cross(tempNormal, vec3(0.0, 0.0, 1.0));
        vec3 c2 = cross(tempNormal, vec3(0.0, 1.0, 0.0));
        
        T = normalize(length(c1) > length(c2) ? c1 : c2);
        B = normalize(cross(tempNormal, T));
    } else {
        // Fallback if no texture coordinates
        T = vec3(1.0, 0.0, 0.0);
        B = vec3(0.0, 1.0, 0.0);
    }
    
    vTBN = mat3(T, B, N);
    
    vec4 viewPosition = view * worldPosition;
    gl_Position = projection * viewPosition;
    
    float distance = length(viewPosition);
    if (distance > fogStart) {
        float fogDistance = distance - fogStart;
        vFogFactor = exp(-pow((fogDistance * fogDensity), fogGradient));
        vFogFactor = clamp(vFogFactor, 0.0, 1.0);
    } else {
        vFogFactor = 1.0;
    }
}