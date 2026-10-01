#version 330

in vec2 fragTexCoord;
in vec4 fragColor;

out vec4 finalColor;

uniform sampler2D texture0;
uniform vec4 colDiffuse;
uniform float uTime;
uniform float uSanctuaryActive;
uniform float uTheme;

float luminance(vec3 color) {
    return dot(color, vec3(0.2126, 0.7152, 0.0722));
}

vec3 brightSample(vec2 uv) {
    vec3 color = texture(texture0, uv).rgb;
    float brightness = luminance(color);
    float crystalBias = max(color.b - max(color.r, color.g) * 0.72, 0.0);
    float mask = smoothstep(0.62, 0.96, brightness) + crystalBias * 0.85;
    return color * clamp(mask, 0.0, 1.0);
}

void main() {
    vec2 uv = fragTexCoord;
    vec2 texel = 1.0 / vec2(textureSize(texture0, 0));
    vec3 base = texture(texture0, uv).rgb;

    // Compact eight-tap bloom. It runs on the GPU and targets the cyan crystals,
    // lanterns, water highlights and warm sun without blurring the whole image.
    vec2 radius = texel * 2.25;
    vec3 bloom = brightSample(uv + vec2( radius.x, 0.0));
    bloom += brightSample(uv + vec2(-radius.x, 0.0));
    bloom += brightSample(uv + vec2(0.0,  radius.y));
    bloom += brightSample(uv + vec2(0.0, -radius.y));
    bloom += brightSample(uv + radius);
    bloom += brightSample(uv - radius);
    bloom += brightSample(uv + vec2(radius.x, -radius.y));
    bloom += brightSample(uv + vec2(-radius.x, radius.y));
    bloom *= 0.125;

    float pulse = 0.94 + 0.06 * sin(uTime * 2.2);
    float bloomStrength = mix(0.16, 0.25 * pulse, uSanctuaryActive);
    vec3 color = base + bloom * bloomStrength;

    // A colorful Captain-Toad-like grade: clean greens, warm stone and cyan magic.
    float gray = luminance(color);
    color = mix(vec3(gray), color, 1.13);
    color = (color - 0.5) * 1.055 + 0.5;
    color *= vec3(1.025, 1.015, 1.04);
    color += vec3(0.025, 0.010, -0.012) * smoothstep(0.55, 1.0, gray);
    color += vec3(-0.010, 0.008, 0.025) * (1.0 - smoothstep(0.08, 0.48, gray));

    if (uTheme > 1.5) {
        color *= vec3(0.92, 1.06, 1.12);
        color += vec3(-0.015, 0.015, 0.035);
    } else if (uTheme > 0.5) {
        color *= vec3(0.96, 0.90, 1.10);
        color += vec3(0.018, -0.010, 0.035) * (1.0 - gray);
    }

    // Very soft vignette keeps the diorama readable without darkening gameplay.
    vec2 centered = uv * 2.0 - 1.0;
    float vignette = 1.0 - dot(centered, centered) * 0.055;
    color *= clamp(vignette, 0.88, 1.0);

    finalColor = vec4(clamp(color, 0.0, 1.0), 1.0) * fragColor * colDiffuse;
}
