struct Arg {
    vec3 p;
    float t;
};

struct MapTransparent {
    vec4 color;
    float d;
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

Arg rotate(vec3 axis, float angle, Arg arg) {
    axis = normalize(axis);
    float s = sin(angle);
    float c = cos(angle);
    float oc = 1.0 - c;

    mat4 rotation_matrix = 
        mat4(oc * axis.x * axis.x + c,           oc * axis.x * axis.y - axis.z * s,  oc * axis.z * axis.x + axis.y * s,  0.0,
                oc * axis.x * axis.y + axis.z * s,  oc * axis.y * axis.y + c,           oc * axis.y * axis.z - axis.x * s,  0.0,
                oc * axis.z * axis.x - axis.y * s,  oc * axis.y * axis.z + axis.x * s,  oc * axis.z * axis.z + c,           0.0,
                0.0,                                0.0,                                0.0,                                1.0);

    vec4 p = rotation_matrix * vec4(arg.p, 1);
    arg.p = p.xyz / p.w;
    return arg;
}

// time transforms

Arg at_t(float t, Arg arg) {
    arg.t -= t;
    return arg;
}

Arg start_at_t(float min_t, Arg arg) {
    arg.t = max(min_t, arg.t);
    return arg;
}

Arg end_at_t(float max_t, Arg arg) {
    arg.t = min(max_t, arg.t);
    return arg;
}

Arg repeat_t(float interval, Arg arg) {
    arg.t = mod(arg.t, interval);
    return arg;
}

Arg map_t(float start_in, float end_in, float start_out, float end_out, Arg arg) {
    arg.t = mix(start_out, end_out, (arg.t - start_in) / (end_in - start_in));
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

float sd_smooth_union(float a, float b, float k)
{
    float h = clamp( 0.5+0.5*(b-a)/k, 0.0, 1.0 );
    return mix( b, a, h ) - k*h*(1.0-h);
}

vec4 csd_union(vec4 a, vec4 b) {
    return a.w < b.w ? a : b;
}

vec4 csd_isect(vec4 a, vec4 b) {
    return a.w > b.w ? a : b;
}

MapTransparent tsd_union(MapTransparent a, MapTransparent b) {
    return a.d < b.d ? a : b;
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
