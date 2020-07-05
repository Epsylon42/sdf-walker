flat in vec3 look;
in vec3 screen_pos;

uniform vec3 cam_pos;
uniform vec3 light;

out vec4 frag_color;

//library
// utils
float vmax(vec3 a) {
    return max(a.x, max(a.y, a.z));
}
float vmin(vec3 a) {
    return min(a.x, min(a.y, a.z));
}


// space transforms
vec3 vat(vec3 shift, vec3 p) {
    return p - shift;
}
vec3 at(float x, float y, float z, vec3 p) {
    return vat(vec3(x,y,z), p);
}

vec3 vrepeat(vec3 size, vec3 p) {
    return mod(p + size/2, size) - size/2;
}
vec3 repeat(float x, float y, float z, vec3 p) {
    return vrepeat(vec3(x,y,z), p);
}


// shape transforms
float sd_union(float a, float b) {
    return min(a, b);
}

float sd_isect(float a, float b) {
    return max(a, b);
}

float sd_diff(float a, float b) {
    return sd_isect(a, -b);
}

float sd_onionize(float thick, float a) {
    return sd_diff(a, a + thick);
}

vec4 csd_union(vec4 a, vec4 b) {
    return a.w < b.w ? a : b;
}

vec4 csd_isect(vec4 a, vec4 b) {
    return a.w > b.w ? a : b;
}


// shapes
float sd_sphere(float r, vec3 p) {
    return length(p) - r;
}

float sd_box(vec3 s, vec3 p)
{
    return vmax(abs(p) - s);
}

float sd_halfspace(vec3 norm, vec3 p) {
    return dot(p, norm);
}

float sd_halfspace_aa(vec3 axis, vec3 p) {
    return vmax(p * axis);
}

float sd_column_aa(vec3 axis, float rad, vec3 p) {
    axis = -(axis - vec3(1));

    return sd_isect(
            sd_halfspace_aa(axis, vat(axis * rad, p)), 
            sd_halfspace_aa(-axis, vat(-axis * rad, p)));
}
//library


float room(vec3 rsize, vec3 p) {
    vec2 dsize = vec2(1.5, 3);
    float thick = 0.1;

    float shell = sd_onionize(thick, sd_box(rsize, at(0,rsize.y,0, p)));

    float door = sd_box(vec3(dsize, thick*2), at(0, dsize.y + thick, -rsize.z + thick, p));

    return sd_diff(shell, door);
}

vec4 map(vec3 p) {
    //vec3 pl_color = vec3(0.74, 0.93, 1.0);
    vec3 rm_color = vec3(0.9, 0.8, 1.0);
    vec3 col_color = vec3(1.0, 0.98, 0.75);

    vec4 rm = vec4(rm_color, room(vec3(5,5,10), at(10, -3, 10, p)));
    vec4 col1 = vec4(rm_color, 
            max(
                sd_halfspace_aa(vec3(0,1,0), at(0,-3,0, p)), 
                sd_column_aa(vec3(0,1,0), 3, at(10, 0, 10, p))));

    vec4 col = vec4(col_color, 
            sd_column_aa(vec3(0,1,0), 50, repeat(1000,0,1000, at(-160,0,5, p))));

    vec3 walkway_scale = vec3(2,0.1,1);
    float walkway = sd_column_aa(vec3(0,0,1), 1, at(-5,0,0, p / walkway_scale)) * vmin(walkway_scale);

    return csd_union(rm, csd_union(col, csd_union(col1, vec4(1,1,1, walkway))));
}

vec3 normal(vec3 p, float d) {
    float dx = map(p + vec3(d, 0, 0)).w - map(p - vec3(d, 0, 0)).w;
    float dy = map(p + vec3(0, d, 0)).w - map(p - vec3(0, d, 0)).w;
    float dz = map(p + vec3(0, 0, d)).w - map(p - vec3(0, 0, d)).w;

    return normalize(vec3(dx, dy, dz));
}

const int steps = 50;
const float max_dist = 2000;
const vec3 ambient = vec3(0.24, 0.09, 0.4);
const float delta = 0.001;
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

vec3 color_at(vec3 pos, vec3 background, float opacity, float shadow) {
    vec3 nlight = -normalize(light);
    float dist = distance(pos, cam_pos);
    vec3 nrm = normal(pos, delta);

    //float diffuse = max(0, dot(nrm, nlight)) * shadow;
    float diffuse = shadow;

    //vec3 specular_vec = reflect(dir, nrm);
    //float specular = max(0, dot(specular_vec, nlight));
    float specular = 0;

    vec3 color = max(ambient / 2, map(pos).xyz * diffuse) + vec3(specular);
    return mix(background, color, opacity);
}

vec3 march(vec3 pos, vec3 dir) {
    //float adaptive_delta = mix(0.0001, delta, smoothstep(0, 10, map(pos).w));
    vec3 nlight = -normalize(light);
    vec3 background = vec3(pow(max(0, dot(dir, nlight)), 10) * 5) + ambient;

    int i = 0;
    for (float t = 0; t < max_dist;) {
        vec4 m = map(pos + dir*t);
        float adaptive_delta = delta;
        if (m.w < delta) {
            float shadow = calc_shadow(pos + dir*t);
            float itershade = smoothstep(35, 5, i) * 0.5 + 0.5;
            return color_at(pos + dir*t, background, smoothstep(max_dist, 0, t) * itershade, shadow);
        }

        t += m.w;
        i += 1;
    }

    return background;
}

void main() {
    vec3 dir = normalize(screen_pos - cam_pos);

    frag_color.xyz = march(cam_pos, dir);
    frag_color.w = 1.0;
}
