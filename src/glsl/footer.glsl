vec3 normal(vec3 p, float d) {
    float dx = map(p + vec3(d, 0, 0)).w - map(p - vec3(d, 0, 0)).w;
    float dy = map(p + vec3(0, d, 0)).w - map(p - vec3(0, d, 0)).w;
    float dz = map(p + vec3(0, 0, d)).w - map(p - vec3(0, 0, d)).w;

    return normalize(vec3(dx, dy, dz));
}
const int steps = 50;
const vec3 ambient = vec3(0.24, 0.09, 0.4);
const float delta = 0.01;
const float shadow_coef = 64;

float calc_shadow(vec3 pos) {
    vec3 nlight = -normalize(light);
    vec3 nrm = normal(pos, delta);

    pos += nrm * delta * 4;

    float closest = 1.0;

    for (float t = 0; t < 100;) {
        float d = map(pos + nlight * t).w;
        closest = min(closest, shadow_coef * d / t);
        if (d < delta) {
            return 0.0;
        }
        t += d;
    }

    return closest;
}

vec3 color_at(vec3 pos, vec3 background, float i, float shadow) {
    float dist = distance(pos, cam_pos);
    vec3 nrm = normal(pos, delta);

    //float diffuse = max(0, dot(nrm, nlight));
    float diffuse = shadow;

    //vec3 specular_vec = reflect(dir, nrm);
    //float specular = max(0, dot(specular_vec, nlight));
    float specular = 0;

    float part_bg = smoothstep(0, steps, i);
    vec3 color = max(ambient / 2, map(pos).xyz * diffuse) + vec3(specular);
    return mix(color, background, part_bg);
}

vec3 march(vec3 pos, vec3 dir) {
    float adaptive_delta = mix(0.0001, delta, smoothstep(0, 10, map(pos).w));

    vec3 nlight = -normalize(light);

    vec3 background = vec3(pow(max(0, dot(dir, nlight)), 10) * 5) + ambient;

    for (int i = 0; i < steps; i++) {
        vec4 m = map(pos);
        if (m.w < adaptive_delta) {
            float shadow = calc_shadow(pos);
            return color_at(pos, background, i, shadow);
        }

        pos += dir * m.w;
    }

    return background;
}

void main() {
    vec3 dir = normalize(screen_pos - cam_pos);

    frag_color.xyz = march(cam_pos, dir);
    frag_color.w = 1.0;
}
