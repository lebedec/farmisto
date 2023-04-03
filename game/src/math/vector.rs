pub trait VectorMath {
    fn neg(self) -> Self;
    fn dot(self, other: Self) -> f32;
    fn invert(self) -> Self;
    fn add(self, other: Self) -> Self;
    fn sub(self, other: Self) -> Self;
    fn mul(self, scalar: f32) -> Self;
    fn div(self, scalar: f32) -> Self;
    fn floor(self) -> Self;
    fn clamp(self, min: Self, max: Self) -> Self;
    fn direction_to(self, other: Self) -> Self;
    fn distance(self, other: Self) -> f32;
    fn length(self) -> f32;
    fn length_squared(self) -> f32;
    fn normalize(self) -> Self;
    fn to_tile(self) -> [usize; 2];
}

impl VectorMath for [f32; 2] {
    #[inline]
    fn neg(self) -> Self {
        [-self[0], -self[1]]
    }

    #[inline]
    fn dot(self, other: Self) -> f32 {
        self[0] * other[0] + self[1] * other[1]
    }

    #[inline]
    fn invert(self) -> Self {
        [-self[0], -self[1]]
    }

    #[inline]
    fn add(self, other: Self) -> Self {
        [self[0] + other[0], self[1] + other[1]]
    }

    #[inline]
    fn sub(self, other: Self) -> Self {
        [self[0] - other[0], self[1] - other[1]]
    }

    #[inline]
    fn mul(self, scalar: f32) -> Self {
        [self[0] * scalar, self[1] * scalar]
    }

    #[inline]
    fn div(self, scalar: f32) -> Self {
        [self[0] / scalar, self[1] / scalar]
    }

    #[inline]
    fn floor(self) -> Self {
        [self[0].floor(), self[1].floor()]
    }

    #[inline]
    fn clamp(self, min: Self, max: Self) -> Self {
        [
            self[0].max(min[0]).min(max[0]),
            self[1].max(min[1]).min(max[1]),
        ]
    }

    #[inline]
    fn direction_to(self, other: Self) -> Self {
        other.sub(self).normalize()
    }

    #[inline]
    fn distance(self, other: Self) -> f32 {
        other.sub(self).length()
    }

    #[inline]
    fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    #[inline]
    fn length_squared(self) -> f32 {
        self.dot(self)
    }

    #[inline]
    fn normalize(self) -> Self {
        if self[0] == 0.0 && self[1] == 0.0 {
            [0.0, 0.0]
        } else {
            let length = self.length();
            [self[0] / length, self[1] / length]
        }
    }

    #[inline]
    fn to_tile(self) -> [usize; 2] {
        [self[0] as usize, self[1] as usize]
    }
}

pub trait FloatMath {
    fn clamp(self, min: Self, max: Self) -> Self;
}

impl FloatMath for f32 {
    #[inline]
    fn clamp(self, min: Self, max: Self) -> Self {
        self.max(min).min(max)
    }
}
