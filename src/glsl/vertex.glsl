in int idx;

uniform float aspect;
uniform float fov;
uniform mat4 cam;
uniform vec3 cam_pos;

flat out vec3 look;
out vec3 screen_pos;

vec2 screen[4] = vec2[](
    vec2(-1, 1),
    vec2(1, 1),
    vec2(1, -1),
    vec2(-1, -1)
);

void main() {
    gl_Position = vec4(screen[idx], 0, 1);

    float dist = 1.0 / tan(fov / 2.0);

    vec4 l = (inverse(cam) * vec4(0,0,1,1));
    look = normalize(l.xyz / l.w);

    vec4 p = inverse(cam) * vec4(screen[idx] * vec2(aspect, 1), dist, 1);
    screen_pos = p.xyz / p.w + cam_pos;
}
