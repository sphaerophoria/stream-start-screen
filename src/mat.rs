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

    pub fn scale(x: f32, y: f32, z: f32) -> Transform {
        let mut transform = Transform::new();
        transform.arr[0][0] = x;
        transform.arr[1][1] = y;
        transform.arr[2][2] = z;
        transform.arr[3][3] = 1.0f32;

        transform
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

    pub fn inverted(&self) -> Transform {
        // Stolen from
        // https://stackoverflow.com/questions/1148309/inverting-a-4x4-matrix
        // The glu convention is [col][row] instead of [row][col]. They're using these as 1d arrays
        // expecting idx = 4 * col + row.
        // We're using [row][col], so invert our array, then cast it to a 1d slice
        let mut m = self.arr;
        for y in 0..4 {
            for x in 0..y {
                let tmp = m[x][y];
                m[x][y] = m[y][x];
                m[y][x] = tmp;
            }
        }

        let m = unsafe { std::slice::from_raw_parts_mut(m.as_mut_ptr() as *mut f32, 16) };

        // Make an output 2d array and cast back to 1d slice for use like in copy pasted code
        let mut out = [[0.0f32; 4]; 4];
        let inv = unsafe { std::slice::from_raw_parts_mut(out.as_mut_ptr() as *mut f32, 16) };

        inv[0] = m[5] * m[10] * m[15] - m[5] * m[11] * m[14] - m[9] * m[6] * m[15]
            + m[9] * m[7] * m[14]
            + m[13] * m[6] * m[11]
            - m[13] * m[7] * m[10];

        inv[4] = -m[4] * m[10] * m[15] + m[4] * m[11] * m[14] + m[8] * m[6] * m[15]
            - m[8] * m[7] * m[14]
            - m[12] * m[6] * m[11]
            + m[12] * m[7] * m[10];

        inv[8] = m[4] * m[9] * m[15] - m[4] * m[11] * m[13] - m[8] * m[5] * m[15]
            + m[8] * m[7] * m[13]
            + m[12] * m[5] * m[11]
            - m[12] * m[7] * m[9];

        inv[12] = -m[4] * m[9] * m[14] + m[4] * m[10] * m[13] + m[8] * m[5] * m[14]
            - m[8] * m[6] * m[13]
            - m[12] * m[5] * m[10]
            + m[12] * m[6] * m[9];

        inv[1] = -m[1] * m[10] * m[15] + m[1] * m[11] * m[14] + m[9] * m[2] * m[15]
            - m[9] * m[3] * m[14]
            - m[13] * m[2] * m[11]
            + m[13] * m[3] * m[10];

        inv[5] = m[0] * m[10] * m[15] - m[0] * m[11] * m[14] - m[8] * m[2] * m[15]
            + m[8] * m[3] * m[14]
            + m[12] * m[2] * m[11]
            - m[12] * m[3] * m[10];

        inv[9] = -m[0] * m[9] * m[15] + m[0] * m[11] * m[13] + m[8] * m[1] * m[15]
            - m[8] * m[3] * m[13]
            - m[12] * m[1] * m[11]
            + m[12] * m[3] * m[9];

        inv[13] = m[0] * m[9] * m[14] - m[0] * m[10] * m[13] - m[8] * m[1] * m[14]
            + m[8] * m[2] * m[13]
            + m[12] * m[1] * m[10]
            - m[12] * m[2] * m[9];

        inv[2] = m[1] * m[6] * m[15] - m[1] * m[7] * m[14] - m[5] * m[2] * m[15]
            + m[5] * m[3] * m[14]
            + m[13] * m[2] * m[7]
            - m[13] * m[3] * m[6];

        inv[6] = -m[0] * m[6] * m[15] + m[0] * m[7] * m[14] + m[4] * m[2] * m[15]
            - m[4] * m[3] * m[14]
            - m[12] * m[2] * m[7]
            + m[12] * m[3] * m[6];

        inv[10] = m[0] * m[5] * m[15] - m[0] * m[7] * m[13] - m[4] * m[1] * m[15]
            + m[4] * m[3] * m[13]
            + m[12] * m[1] * m[7]
            - m[12] * m[3] * m[5];

        inv[14] = -m[0] * m[5] * m[14] + m[0] * m[6] * m[13] + m[4] * m[1] * m[14]
            - m[4] * m[2] * m[13]
            - m[12] * m[1] * m[6]
            + m[12] * m[2] * m[5];

        inv[3] = -m[1] * m[6] * m[11] + m[1] * m[7] * m[10] + m[5] * m[2] * m[11]
            - m[5] * m[3] * m[10]
            - m[9] * m[2] * m[7]
            + m[9] * m[3] * m[6];

        inv[7] = m[0] * m[6] * m[11] - m[0] * m[7] * m[10] - m[4] * m[2] * m[11]
            + m[4] * m[3] * m[10]
            + m[8] * m[2] * m[7]
            - m[8] * m[3] * m[6];

        inv[11] = -m[0] * m[5] * m[11] + m[0] * m[7] * m[9] + m[4] * m[1] * m[11]
            - m[4] * m[3] * m[9]
            - m[8] * m[1] * m[7]
            + m[8] * m[3] * m[5];

        inv[15] = m[0] * m[5] * m[10] - m[0] * m[6] * m[9] - m[4] * m[1] * m[10]
            + m[4] * m[2] * m[9]
            + m[8] * m[1] * m[6]
            - m[8] * m[2] * m[5];

        let mut det = m[0] * inv[0] + m[1] * inv[4] + m[2] * inv[8] + m[3] * inv[12];

        det = 1.0 / det;

        for i in 0..16 {
            inv[i] *= det;
        }

        // And put it back [row][col]
        for y in 0..4 {
            for x in 0..y {
                let tmp = out[x][y];
                out[x][y] = out[y][x];
                out[y][x] = tmp;
            }
        }
        Transform { arr: out }
    }

    pub fn perspective(fov: f32, near: f32, far: f32) -> Transform {
        // Perspective is applied by taking the fov in each dimension and splitting it into a right
        // angle triangle.
        //
        // Imagine we are trying to draw the object in the following top down fov
        //
        // \    |    /
        //  \ o |   /
        //   \  |  /
        //    \ | /
        //     \|/
        //
        // If we were to stretch the FoV into opengl's cube, we essentially have to make the
        // diagonal lines parallel at -1 and 1. i.e. at all points the diagnoal line is 1
        //
        // If the vertical line is our distance to an object in Z, we can say that the object
        // should be offset by some distance relative to the diagnoal line tan(theta / 2) will give
        // us opposite / adjacent, e.g. X / Z
        //
        // We know that we want X to to be 1.0, so we want to have something like
        // u = x / X where u is the output coord, x is our x, and X is the distance to the diagonal
        //
        // x / tan(theta) gives us zx/X, and then we divide by z again later (see 4th row of output
        // matrix) to get the final answer
        //
        // We can also think of the answer as being some constant / Z, as we know X scales linearly
        // with Z for any given FoV
        let xy = 1.0 / f32::tan(fov / 2.0);

        // Now we have to deal with the near/far plane. We know that we want to make the following
        // mappings
        // [near, far] -> [-1, 1]
        //
        // Lets start by trying to find coefficients that will map these values linearly. Using a
        // typical line formula
        // w = mz + b
        //
        // But because we set the homogeneous coordinate to z, we need to multiply our desired
        // outputs by z to offset
        // So we need to map [near, far] -> [-near, far], as -near/near = -1, and far/far = 1
        //
        // We have two unknowns (m and b) and two equations
        // f = mf + b
        // -n = mn + b
        //
        // If we isolate b so we can express m in terms of n and f
        // f-fm = b
        // -n- mn = b
        // f = mf - n - mn
        // f + n = m(f-n)
        // m = (f+n)/(f-n)
        //
        // And substituting that back in find b
        // b = -n -mn
        // -n - n(f+n)/(f-n)
        // -n(f-n)/(f-n) - n (f+n)/(f-n)
        // -nf - nf
        // -2nf / (f-n)
        //
        // Now we just plug m and b into the 3rd and 4th columsn in our matrix and we're good to
        // go, as column 3 will automatically be multiplied by Z
        //
        // Also note that this ends up with a w ~ 1/z relationship, as our output space is divided
        // by z entirely. This is great because we end up with a Z value that is interpolated
        // linearly in screen space
        // (https://alexsabourindev.wordpress.com/2019/08/27/a-quest-towards-intuition-why-is-depth-interpolated-as-1-z/)

        let z_dist = far - near;
        let arr = [
            [xy, 0.0, 0.0, 0.0],
            [0.0, xy, 0.0, 0.0],
            [0.0, 0.0, (near + far) / z_dist, -2.0 * near * far / z_dist],
            [0.0, 0.0, 1.0, 0.0],
        ];

        Transform { arr }
    }
}

impl std::ops::Mul<&Self> for Transform {
    type Output = Self;

    fn mul(self, rhs: &Self) -> Self {
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
impl std::ops::Mul<Self> for Transform {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        self.mul(&rhs)
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
