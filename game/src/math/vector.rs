pub trait VectorMath {
    fn dot(self, other: Self) -> f32;
    fn invert(self) -> Self;
    fn add(self, other: Self) -> Self;
    fn sub(self, other: Self) -> Self;
    fn mul(self, scalar: f32) -> Self;
    fn direction(self, other: Self) -> Self;
    fn distance(self, other: Self) -> f32;
    fn length(self) -> f32;
    fn length_squared(self) -> f32;
    fn normalize(self) -> Self;
}

impl VectorMath for [f32; 2] {
    #[inline]
    fn dot(self, other: Self) -> f32 {
        self[0] * other[0] + self[1] + other[1]
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
    fn direction(self, other: Self) -> Self {
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
        self[0] * self[0] + self[1] * self[1]
    }

    #[inline]
    fn normalize(self) -> Self {
        if self[0] + self[1] == 0.0 {
            [0.0, 0.0]
        } else {
            let length = self.length();
            [self[0] / length, self[1] / length]
        }
    }
}
