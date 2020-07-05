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

mat4 rotate_z(float theta) {
    float c = cos(theta);
    float s = sin(theta);

    return mat4(
        vec4(c, s, 0, 0),
        vec4(-s, c, 0, 0),
        vec4(0, 0, 1, 0),
        vec4(0, 0, 0, 1)
    );
}

vec3 mult(mat4 mat, vec3 vec) {
    vec4 intermediate = mat * vec4(vec, 1);
    return intermediate.xyz / intermediate.w;
}

//transparency
vec4 tp_color(mat4 tp) {
    return tp[0];
}

float tp_depth(mat4 tp) {
    return tp[1].w;
}
//library

float sd_ladder(float rad, float width, vec3 p) {
    mat4 rot = rotate_z(3.1415 / 4);

    vec3 pr = mult(rot, at(0,rad,0, p));
    pr = repeat(0,rad*2*sqrt(2),0, pr);
    pr = mult(inverse(rot), pr);

    float ladder = sd_column_aa(vec3(0,0,1), rad, pr);

    float limiter = sd_isect(
        sd_halfspace_aa(vec3(0,0,1), at(0,0,width*2, p)),
        sd_halfspace_aa(vec3(0,0,-1), at(0,0,-width*2, p))
            );
    
    return sd_isect(ladder, limiter);
}

float ladder_with_platform(float rad, float width, vec3 p) {
    float hs = sd_halfspace(vec3(0,1,0), p);

    float l1 = sd_isect(sd_ladder(rad, width, at(-width+rad,0,0, p)), hs);
    float l2 = sd_diff(sd_ladder(rad, width, at(width+rad*3,0,0, p)), hs + rad * 2);
    float platform = sd_box(vec3(width*2,rad,width*2), at(0,-rad,0, p));

    return sd_union(platform, sd_union(l1, l2));
}

float room(vec3 rsize, vec3 p) {
    vec2 dsize = vec2(1.5, 3);
    float thick = 0.5;

    float shell = sd_onionize(thick, sd_box(rsize, at(0,rsize.y,0, p)));

    float door = sd_box(vec3(dsize, thick*2), at(0, dsize.y + thick, -rsize.z + thick, p));

    return sd_diff(shell, door);
}

float prefab_glass_column(vec3 p) {
    return sd_column_aa(vec3(1,0,0), 2, at(0,5,0, p));
}

float prefab_room(vec3 p) {
    return sd_diff(room(vec3(5,5,10), p), prefab_glass_column(p) - 0.1);
}

vec4 map(vec3 p) {
    //vec3 pl_color = vec3(0.74, 0.93, 1.0);
    vec3 rm_color = vec3(0.9, 0.8, 1.0);
    vec3 col_color = vec3(1.0, 0.98, 0.75);

    vec4 rm = vec4(rm_color, prefab_room(at(10, -3, 10, p)));
    vec4 col1 = vec4(rm_color, 
            max(
                sd_halfspace_aa(vec3(0,1,0), at(0,-3,0, p)), 
                sd_column_aa(vec3(0,1,0), 3, at(10, 0, 10, p))));

    vec4 col = vec4(col_color, 
            sd_column_aa(vec3(0,1,0), 50, repeat(1000,0,1000, at(-160,0,5, p))));

    //vec3 walkway_scale = vec3(2,0.1,1);
    //vec4 walkway = vec4(1,1,1, sd_column_aa(vec3(0,0,1), 1, at(-5,100,0, p / walkway_scale))) * vmin(walkway_scale);

    //float ladder_platform = sd_halfspace(vec3(0,1,0), at(0,-3,0, p));
    //float ladder1 = sd_isect(sd_ladder(0.5, 1, at(10,-3,-2, p)), ladder_platform);
    //float ladder2 = sd_diff(sd_ladder(0.5, 1, at(12,-3,-2, p)), ladder_platform + 1);

    //vec4 ladder_shape = vec4(1,1,1, sd_union(ladder1, ladder2));
    //vec4 ladder = vec4(1,1,1, ladder_with_platform(0.5, 1, at(10,-3,-2, p)));

    return csd_union(rm, csd_union(col, csd_union(col1, vec4(0,0,0, 1.0/0.0))));
}

mat4 map_transparent(vec3 p) {
    p = at(10, -3, 10, p);
    float glass = sd_isect(room(vec3(5,5,10), p), prefab_glass_column(p) - 0.1);

    return mat4(
            vec4(0.7, 0.7, 1.0, 1.0),
            vec4(0,0,0, glass),
            vec4(0),
            vec4(0)
            );
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
const float delta = 0.01;
const float shadow_coef = 64;

struct Shadow {
    vec4 color;
    float shadow;
};

vec3 color_at(vec3 pos, vec3 background, float opacity, Shadow shadow) {
    vec3 nlight = -normalize(light);
    float dist = distance(pos, cam_pos);
    vec3 nrm = normal(pos, delta);

    vec3 diffuse = mix(map(pos).xyz, shadow.color.xyz, shadow.color.w) * shadow.shadow;

    vec3 color = max(ambient / 2, diffuse);
    return mix(background, color, opacity);
}


Shadow calc_shadow(vec3 pos) {
    vec3 nlight = -normalize(light);
    vec3 nrm = normal(pos, delta);

    pos += nrm * delta * 4;

    float closest = 1.0;

    vec4 collected_color = vec4(0);
    for (float t = 0; t < 100;) {
        float od = map(pos + nlight * t).w;
        if (od < delta) {
            return Shadow(vec4(0), 0);
        }
        closest = min(closest, shadow_coef * od / t);

        mat4 transparent = map_transparent(pos + nlight * t);
        float td = tp_depth(transparent);

        if (td < delta) {
            collected_color = mix(collected_color, tp_color(transparent), abs(td));
            t += max(abs(td), delta);
        } else {
            t += max(od, delta);
        }
    }

    return Shadow(collected_color, closest);
}

vec3 march(vec3 pos, vec3 dir) {
    vec3 nlight = -normalize(light);
    vec3 background = vec3(pow(max(0, dot(dir, nlight)), 10) * 5) + ambient;

    int i = 0;
    vec4 collected_color = vec4(0.0001);
    for (float t = 0; t < max_dist;) {
        vec4 m = map(pos + dir*t);
        float od = m.w;

        mat4 transparent = map_transparent(pos + dir*t);
        float td = tp_depth(transparent);

        if (td < delta) {
            collected_color = mix(collected_color, tp_color(transparent), abs(td));
            t += max(abs(td), delta);
        } else if (od < delta) {
            float itershade = smoothstep(35, 5, i) * 0.5 + 0.5;
            float distshade = smoothstep(max_dist, 0, t);
            vec3 solid_color = color_at(pos + dir*t, background, distshade * itershade, calc_shadow(pos + dir*t));
            return mix(solid_color, collected_color.xyz, min(1, collected_color.w));
        } else {
            t += max(min(od, td), delta);
        }

        i += 1;
    }

    return mix(background, collected_color.xyz, collected_color.w);
}

void main() {
    vec3 dir = normalize(screen_pos - cam_pos);

    frag_color.xyz = march(cam_pos, dir);
    frag_color.w = 1.0;
}
