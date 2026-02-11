SCALE = 1 << 32
MAX_I64 = (1 << 63) - 1
MIN_I64 = -(1 << 63)

DT = SCALE // 50
FORCE = SCALE * 10
FRICTION = SCALE // 100
GRAVITY = (SCALE * 49) // 5
ANGLE_STIFFNESS = SCALE
ANGLE_DAMPING = SCALE // 10
POS_LIMIT = (SCALE * 12) // 5
ANGLE_LIMIT = (SCALE * 209) // 1000


def clamp_i64(value):
    if value > MAX_I64:
        return MAX_I64
    if value < MIN_I64:
        return MIN_I64
    return value


def f_add(a, b):
    return clamp_i64(a + b)


def f_sub(a, b):
    return clamp_i64(a - b)


def f_mul(a, b):
    return clamp_i64((a * b) >> 32)


def f_abs(value):
    return -value if value < 0 else value


def splitmix64(x):
    x = (x + 0x9e3779b97f4a7c15) & 0xFFFFFFFFFFFFFFFF
    z = x
    z = ((z ^ (z >> 30)) * 0xbf58476d1ce4e5b9) & 0xFFFFFFFFFFFFFFFF
    z = ((z ^ (z >> 27)) * 0x94d049bb133111eb) & 0xFFFFFFFFFFFFFFFF
    return z ^ (z >> 31)


def seed_to_fixed(seed, shift):
    bits = (seed >> shift) & 0xFFFF
    centered = bits - 0x8000
    return clamp_i64(centered << 16)


def reset(seed):
    base = splitmix64(seed)
    return {
        "x": seed_to_fixed(base, 0),
        "v": seed_to_fixed(base, 16),
        "theta": seed_to_fixed(base, 32),
        "omega": seed_to_fixed(base, 48),
    }


def observe(state):
    return (state["x"], state["v"], state["theta"], state["omega"])


def is_done(state):
    return f_abs(state["x"]) > POS_LIMIT or f_abs(state["theta"]) > ANGLE_LIMIT


def step(state, action):
    if action not in (-1, 1):
        raise ValueError("action must be -1 or 1")
    act = action * SCALE

    accel = f_sub(f_mul(act, FORCE), f_mul(state["v"], FRICTION))
    state["v"] = f_add(state["v"], f_mul(accel, DT))
    state["x"] = f_add(state["x"], f_mul(state["v"], DT))

    ang_accel = f_sub(
        f_sub(f_mul(f_mul(act, FORCE), ANGLE_STIFFNESS), f_mul(state["theta"], GRAVITY)),
        f_mul(state["omega"], ANGLE_DAMPING),
    )
    state["omega"] = f_add(state["omega"], f_mul(ang_accel, DT))
    state["theta"] = f_add(state["theta"], f_mul(state["omega"], DT))


if __name__ == "__main__":
    seed = 42
    actions = [1, 1, -1, 1, -1, -1, 1, 1, 1, -1, 1, -1]
    state = reset(seed)
    for action in actions:
        if is_done(state):
            break
        step(state, action)
    print("smoke_ok", observe(state))
