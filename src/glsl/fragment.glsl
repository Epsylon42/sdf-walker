flat in vec3 look;
in vec3 screen_pos;

uniform vec3 cam_pos;
uniform vec3 light;

out vec4 frag_color;

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

vec4 csdf_and(vec4 a, vec4 b) {
    return a.w < b.w ? a : b;
}

float vmax(vec3 a) {
    return max(a.x, max(a.y, a.z));
}

float sphere(float r, vec3 p) {
    return length(p) - r;
}

float plane(vec3 norm, vec3 p) {
    return dot(p, norm);
}

float box(vec3 s, vec3 p)
{
    return vmax(abs(p) - s);
}

float column(float rad, vec3 p) {
    p.xz += rad / 2;

    float x = max(p.x - rad, -p.x);
    float z = max(p.z - rad, -p.z);

    return max(x, z);
}

float room(vec3 p) {
    float thick = 0.1;

    float outer = box(vec3(5,5,10), p);
    float inner = box(vec3(5,5,10)-(thick*2), vat(vec3(thick), p));
    float door = box(vec3(1.5, 3, thick*3), vat(vec3(0,-1.7,-10+thick), p));

    float bx = max(outer, -inner);
    return max(bx, -door);
}

vec4 map(vec3 p) {
    //vec3 pl_color = vec3(0.74, 0.93, 1.0);
    vec3 rm_color = vec3(0.9, 0.8, 1.0);
    vec3 col_color = vec3(1.0, 0.98, 0.75);

    vec4 rm = vec4(rm_color, room(at(10, 3, 10, p)));
    vec4 col1 = vec4(rm_color, max(plane(vec3(0,1,0), at(0,-2,0, p)), column(7, at(10, 0, 10, p))));
    vec4 col = vec4(col_color, column(50, repeat(1000,0,1000, at(-80,0,5, p))));

    return csdf_and(rm, csdf_and(col, col1));
}

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
    vec3 nlight = -normalize(light);

    vec3 background = vec3(pow(max(0, dot(dir, nlight)), 10) * 5) + ambient;

    for (int i = 0; i < steps; i++) {
        vec4 m = map(pos);
        if (m.w < delta) {
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
