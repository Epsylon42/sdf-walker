struct Arg {
    vec3 p;
    float t;
};

// utils
float vmax(vec3 a) {
    return max(a.x, max(a.y, a.z));
}
float vmin(vec3 a) {
    return min(a.x, min(a.y, a.z));
}


// space transforms
Arg vat(vec3 shift, Arg arg) {
    arg.p -= shift;
    return arg;
}
Arg at(float x, float y, float z, Arg arg) {
    return vat(vec3(x,y,z), arg);
}

Arg vscale(vec3 sc, Arg arg) {
    arg.p /= sc;
    return arg;
}
Arg scale(float x, float y, float z, Arg arg) {
    return vscale(vec3(x,y,z), arg);
}
Arg uscale(float sc, Arg arg) {
    arg.p /= sc;
    return arg;
}

Arg vrepeat(vec3 size, Arg arg) {
    arg.p = mod(arg.p + size/2, size) - size/2;
    return arg;
}
Arg repeat(float x, float y, float z, Arg arg) {
    return vrepeat(vec3(x,y,z), arg);
}

// time transforms

Arg at_t(float t, Arg arg) {
    arg.t -= t;
    return arg;
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
float sd_sphere(float r, Arg arg) {
    return length(arg.p) - r;
}

float sd_box(vec3 s, Arg arg)
{
    return vmax(abs(arg.p) - s);
}

float sd_halfspace(vec3 norm, Arg arg) {
    return dot(arg.p, norm);
}

float sd_halfspace_aa(vec3 axis, Arg arg) {
    return vmax(arg.p * axis);
}

float sd_column_aa(vec3 axis, float rad, Arg arg) {
    axis = -(axis - vec3(1));

    return sd_isect(
            sd_halfspace_aa(axis, vat(axis * rad, arg)), 
            sd_halfspace_aa(-axis, vat(-axis * rad, arg)));
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
