use std::convert::From;

#[derive(Debug, Copy, Clone)]
pub struct Vec3([f32; 3]);

impl Vec3 {
    pub fn x(&self) -> f32 {
        self.0[0]
    }

    pub fn y(&self) -> f32 {
        self.0[1]
    }

    pub fn z(&self) -> f32 {
        self.0[2]
    }

    pub fn length(&self) -> f32 {
        let l_2 = self.0.iter().map(|v| v * v).sum();
        f32::sqrt(l_2)
    }

    pub fn normalized(&self) -> Vec3 {
        let l = self.length();

        [self.x() / l, self.y() / l, self.z() / l].into()
    }
}

impl From<[f32; 3]> for Vec3 {
    fn from(value: [f32; 3]) -> Self {
        Self(value)
    }
}

impl std::ops::Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut ret = [0.0f32; 3];
        for i in 0..3 {
            ret[i] = self.0[i] - rhs.0[i]
        }

        ret.into()
    }
}

#[allow(unused)]
pub enum Axis {
    X,
    Y,
    Z,
}

#[derive(Debug)]
pub struct Transform {
    pub arr: [[f32; 4]; 4],
}

impl Transform {
    pub fn new() -> Transform {
        Transform {
            arr: [[0.0f32; 4]; 4],
        }
    }

    pub fn identity() -> Transform {
        let arr = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];

        Transform { arr }
    }

    pub fn from_translation(x: f32, y: f32, z: f32) -> Transform {
        let mut transform = Transform::identity();
        transform.arr[0][3] = x;
        transform.arr[1][3] = y;
        transform.arr[2][3] = z;
        transform
    }

    pub fn from_axis_angle(angle: f32, axis: Axis) -> Transform {
        let cx = f32::cos(angle);
        let sx = f32::sin(angle);
        let mut transform = Transform::identity();
        match axis {
            Axis::X => {
                transform.arr[1][1] = cx;
                transform.arr[1][2] = -sx;
                transform.arr[2][1] = sx;
                transform.arr[2][2] = cx;
            }
            Axis::Y => {
                transform.arr[0][0] = cx;
                transform.arr[0][2] = -sx;
                transform.arr[2][0] = sx;
                transform.arr[2][2] = cx;
            }
            Axis::Z => {
                transform.arr[0][0] = cx;
                transform.arr[0][1] = -sx;
                transform.arr[1][0] = sx;
                transform.arr[1][1] = cx;
            }
        }
        transform
    }
}

impl std::ops::Mul for Transform {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let mut output = Transform::new();
        for y in 0..4 {
            for x in 0..4 {
                for i in 0..4 {
                    // FIXME: Duplciated
                    output.arr[y][x] += self.arr[y][i] * rhs.arr[i][x]
                }
            }
        }
        output
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_simple_mul() {
        let a = Transform {
            arr: [
                [1.0f32, 2.0f32, 3.0f32, 4.0f32],
                [5.0f32, 6.0f32, 7.0f32, 8.0f32],
                [9.0f32, 0.0f32, 1.0f32, 2.0f32],
                [3.0f32, 4.0f32, 5.0f32, 6.0f32],
            ],
        };
        let b = Transform {
            arr: [
                [2.0f32, 3.0f32, 4.0f32, 5.0f32],
                [6.0f32, 7.0f32, 8.0f32, 9.0f32],
                [10.0f32, 1.0f32, 2.0f32, 3.0f32],
                [4.0f32, 5.0f32, 6.0f32, 7.0f32],
            ],
        };
        let c = a * b;

        let expected: [[f32; 4]; 4] = [
            [60.0f32, 40.0f32, 50.0f32, 60.0f32],
            [148.0f32, 104.0f32, 130.0f32, 156.0f32],
            [36.0f32, 38.0f32, 50.0f32, 62.0f32],
            [104.0f32, 72.0f32, 90.0f32, 108.0f32],
        ];

        for y in 0..4 {
            for x in 0..4 {
                assert!((expected[y][x] - c.arr[y][x]).abs() < 0.001);
            }
        }
    }
}
