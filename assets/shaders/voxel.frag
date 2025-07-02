#version 330 core

in vec2 vTexCoord;
in float vFogFactor;
in vec3 vNormal;
in vec3 vWorldPos;
in mat3 vTBN;
out vec4 FragColor;

uniform sampler2D atlas;
uniform sampler2D normalMap;
uniform float usingAlpha;
uniform float useNormalMap = 1.0;
uniform vec3 fogColor = vec3(0.61, 0.78, 1.0);

// Lighting uniforms
uniform vec3 sunDirection = normalize(vec3(-0.5, -1.0, -0.5));
uniform vec3 lightColor = vec3(1.0, 0.96, 0.91);
uniform vec3 ambientColor = vec3(0.75, 0.97, 1.0);
uniform float lightIntensity = 1.2;

// Phong-specific uniforms
uniform float specularStrength = 0.5;    // Adjust specular intensity
uniform float shininess = 32.0;          // Adjust highlight sharpness
uniform vec3 viewPos;                    // Camera position

vec3 calculatePhongLighting(vec3 normal, vec3 color) {
    vec3 norm = normalize(normal);
    vec3 viewDir = normalize(viewPos - vWorldPos);
    vec3 reflectDir = reflect(sunDirection, norm);

    // Ambient
    vec3 ambient = ambientColor * color;

    // Diffuse
    float diff = max(dot(norm, -sunDirection), 0.0);
    vec3 diffuse = lightColor * diff * color * lightIntensity;

    // Specular (Phong)
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), shininess);
    vec3 specular = lightColor * spec * specularStrength;

    return ambient + diffuse + specular;
}

void main() {
    vec4 texColor = texture(atlas, vTexCoord);
    
    if (usingAlpha == 1.0 && texColor.a < 0.5) {
        discard;
    }
    
    // Normal mapping
    vec3 normal = vNormal;
    if (useNormalMap == 1.0) {
        vec3 normalMapValue = texture(normalMap, vTexCoord).rgb * 2.0 - 1.0;
        normal = normalize(vTBN * normalMapValue);
    }
    
    // Apply Phong lighting
    vec3 litTexColor = calculatePhongLighting(normal, texColor.rgb);
    vec3 litFogColor = calculatePhongLighting(normal, fogColor);
    
    // Mix with fog
    FragColor = mix(vec4(litFogColor, texColor.a), vec4(litTexColor, texColor.a), vFogFactor);
}