// Common functions for smoke simulation

fn hash12(p: vec2<f32>) -> f32 {
	let p3 = fract(vec3<f32>(p.xyx) * vec3<f32>(0.1031));
    p3 = p3 + dot(p3, p3.yzx + vec3<f32>(33.33));
    return fract((p3.x + p3.y) * p3.z);
}

fn hash41(p: f32) -> vec4<f32> {
	let p4 = fract(vec4<f32>(p) * vec4<f32>(0.1031, 0.1030, 0.0973, 0.1099));
    p4 = p4 + dot(p4, p4.wzxy + vec4<f32>(33.33));
    return fract((p4.xxyz + p4.yzzw) * p4.zywx);
}

fn rotate(angle: f32, radius: f32) -> vec2<f32> {
    return vec2<f32>(cos(angle), -sin(angle)) * radius;
}

fn dist2(v: vec3<f32>) -> f32 {
    return dot(v, v);
}
