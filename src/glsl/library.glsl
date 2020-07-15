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

vec3 vscale(vec3 sc, vec3 p) {
    return p / sc;
}
vec3 scale(float x, float y, float z, vec3 p) {
    return vscale(vec3(x,y,z), p);
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

struct Shadow {
    vec3 mask;
    float shadow;
};

struct MapTransparent {
    vec4 color;
    float d;
};

vec3 apply_mask(vec3 color, vec3 mask) {
    if (length(1 - mask) > 1) {
        mask = normalize(mask);
    }
    return color * mask;
}
