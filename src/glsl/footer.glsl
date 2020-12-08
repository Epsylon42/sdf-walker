vec4 map(vec3 p) {
    return map_impl(Arg(p, time));
}

MapTransparent map_transparent(vec3 p) {
    return map_transparent_impl(Arg(p, time));
}

vec3 normal(vec3 p, float d) {
    float dx = map(p + vec3(d, 0, 0)).w - map(p - vec3(d, 0, 0)).w;
    float dy = map(p + vec3(0, d, 0)).w - map(p - vec3(0, d, 0)).w;
    float dz = map(p + vec3(0, 0, d)).w - map(p - vec3(0, 0, d)).w;

    vec3 grad = vec3(dx, dy, dz);
    if (length(grad) == 0)
    {
        vec3(0);
    }
    else
    {
        return normalize(grad);
    }
}

const int steps = 50;
const float max_dist = 2000;
const vec3 sky = vec3(0.24, 0.09, 0.4);
const vec3 ambient = sky * 0.9;
const float delta = 0.01;
const float shadow_coef = 64;

struct Shadow {
    vec3 mask;
    float shadow;
};

vec3 apply_mask(vec3 color, vec3 mask) {
    if (length(1 - mask) > 1) {
        mask = normalize(mask);
    }
    return color * mask;
}

float ambient_occlusion(vec3 pos, vec3 nrm) {
    const float max_dist = 10;
    const int steps = 6;

    float factor = 0;
    for (int i = 1; i < steps + 1; i++) {
        float frac = float(i) / float(steps);
        frac *= frac;

        float dist = max_dist * frac;
        dist = dist + pow(sin((pos.x + pos.y + pos.z) / 3 + dist), 2.0);
        vec3 p = pos + nrm * dist;

        factor += (map(p).w) / dist / steps;
    }

    return (factor + 0.4) / 1.4;
}

vec3 color_at(vec3 pos, vec3 background, float opacity, Shadow shadow) {
    vec3 nlight = -normalize(light);
    float dist = distance(pos, cam_pos);
    vec3 nrm = normal(pos, delta);

    vec3 diffuse = map(pos).xyz;
    vec3 reflected = apply_mask(diffuse, shadow.mask) * shadow.shadow;

    return mix(background, max(ambient * diffuse * ambient_occlusion(pos, nrm), reflected), opacity);
}


Shadow calc_shadow(vec3 pos) {
    vec3 nlight = -normalize(light);
    vec3 nrm = normal(pos, delta);

    pos += normalize(screen_pos - pos) * delta * 4;
    //pos += nrm * delta * 4;

    float closest = 1.0;

    vec3 mask = vec3(1);

    for (float t = 0; t < 100;) {
        float od = map(pos + nlight * t).w;
        if (od < delta) {
            return Shadow(vec3(1), 0);
        }
        closest = min(closest, shadow_coef * od / t);

        MapTransparent transparent = map_transparent(pos + nlight * t);

        if (transparent.d < delta) {
            //mask += transparent.color.xyz * transparent.color.w * abs(transparent.d);
            //mask = pow(mask, transparent.color.xyz * transparent.color.w / abs(transparent.d));
            mask -= transparent.color.xyz * transparent.color.w * abs(transparent.d);
            t += max(abs(transparent.d), delta);
        } else {
            t += max(od, delta);
        }
    }

    return Shadow(mask, closest);
}

vec3 march(vec3 pos, vec3 dir) {
    vec3 nlight = -normalize(light);
    vec3 background = vec3(pow(max(0, dot(dir, nlight)), 10) * 5) + sky;

    for (int tries = 0; tries < 3; tries += 1) {
        int i = 0;
        vec3 mask = vec3(1);
        bool continue_outer = false;
        for (float t = 0; t < max_dist;) {
            vec4 m = map(pos + dir*t);
            float od = m.w;

            MapTransparent transparent = map_transparent(pos + dir*t);

            if (transparent.d < delta) {
                mask -= transparent.color.xyz * transparent.color.w * abs(transparent.d);
                t += max(abs(transparent.d), delta);
            } else if (od < delta) {
                if (normal(pos + dir*t, delta) == vec3(0) && tries < 3) {
                    continue_outer = true;
                    break;
                }

                float itershade = smoothstep(35, 5, i) * 0.5 + 0.5;
                float distshade = smoothstep(max_dist, 0, t);
                vec3 solid_color = color_at(pos + dir*t, background, distshade * itershade, calc_shadow(pos + dir*t));
                return apply_mask(solid_color, mask);
            } else {
                t += max(min(od, transparent.d), delta) * (1 - 0.1 * tries);
            }

            i += 1;
        }

        if (continue_outer) {
            continue;
        }

        return apply_mask(background, mask);
    }
}

void main() {
    vec3 dir = normalize(screen_pos - cam_pos);

    frag_color.xyz = march(cam_pos, dir);
    frag_color.w = 1.0;
}
