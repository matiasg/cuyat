use nalgebra::{Dyn, Matrix, Matrix3, OMatrix, SVector, UnitQuaternion, U3};

type SkyMat = OMatrix<f32, Dyn, U3>;
pub type Star = SVector<f32, 3>;
type Position = SVector<f32, 3>;
type Fpp = SVector<f32, 2>; // Focal Plane Point
pub type FPStars = Vec<Fpp>;

#[derive(Clone)]
pub struct Sky {
    stars: Vec<Star>,
}

impl Sky {
    pub fn from(stars: Vec<Star>) -> Self {
        Self { stars }
    }

    pub fn len(&self) -> usize {
        self.stars.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.stars.is_empty()
    }

    pub fn seen_from(&self, pos: Position) -> Self {
        Self {
            stars: self.stars.iter().map(|&s| s - pos).collect(),
        }
    }

    pub fn with_attitude(&self, q: UnitQuaternion<f32>) -> Self {
        Self {
            stars: self.stars.iter().map(|&s| q * s).collect(),
        }
    }

    pub fn random_with_stars(n: usize) -> Self {
        let stars: Vec<Star> = (0..n).map(|_| Star::new_random() * 10.0).collect();
        let sky = Self { stars };
        sky.seen_from(Star::new(5.0, 5.0, 5.0))
    }
}

#[derive(Clone)]
pub struct FoV {
    half_fov_x: f32,
    half_fov_y: f32,
}

impl FoV {
    pub fn new(half_fov_x: f32, half_fov_y: f32) -> Self {
        Self {
            half_fov_x,
            half_fov_y,
        }
    }
    pub fn project(&self, star: Star) -> Fpp {
        Fpp::new(
            star[0] / star[2] / self.half_fov_x,
            star[1] / star[2] / self.half_fov_y,
        )
    }

    pub fn project_sky(&self, sky: Sky) -> FPStars {
        sky.stars.iter().map(|&s| self.project(s)).collect()
    }

    pub fn to_screen(&self, star: Star, maxx: u8, maxy: u8) -> Option<(u8, u8)> {
        let fpp = self.project(star);
        let x = ((fpp[0] + 1.0) / 2.0 * (maxx as f32)).floor() as u8;
        let y = ((fpp[1] + 1.0) / 2.0 * (maxy as f32)).floor() as u8;

        if inside(x, 0, maxx) && inside(y, 0, maxy) {
            Some((x, y))
        } else {
            None
        }
    }

    pub fn project_sky_to_screen(&self, sky: Sky, maxx: u8, maxy: u8) -> Vec<Option<(u8, u8)>> {
        sky.stars
            .iter()
            .map(|&s| self.to_screen(s, maxx, maxy))
            .collect()
    }

    pub fn with_angles(x_rad: f32, y_rad: f32) -> Self {
        Self {
            half_fov_x: x_rad.tan() / 2.0,
            half_fov_y: y_rad.tan() / 2.0,
        }
    }
}

fn inside(x: u8, minval: u8, maxval: u8) -> bool {
    minval <= x && x <= maxval
}

#[cfg(test)]
mod test {
    use std::f32::consts::PI;

    use nalgebra::UnitQuaternion;

    use crate::{FoV, Fpp, Position, Sky, Star};

    fn stars() -> Vec<Star> {
        vec![Star::new(0.0, 1.0, 2.0), Star::new(3.0, 4.0, 5.0)]
    }
    #[test]
    fn test_sky() {
        let sky = Sky::from(stars());
        assert_eq!(sky.len(), 2);
        let pos = Position::new(-1.0, -2.0, -3.0);
        let from_pos = sky.seen_from(pos);
        assert_eq!(from_pos.len(), 2);
        assert_eq!(
            from_pos.stars,
            vec![Star::new(1.0, 3.0, 5.0), Star::new(4.0, 6.0, 8.0)]
        );
        let q = UnitQuaternion::from_euler_angles(0.0, 0.0, PI / 2.0);
        let rotated = from_pos.with_attitude(q);
        assert_eq!(rotated.len(), 2);
        assert!((rotated.stars[0] - Star::new(-3.0, 1.0, 5.0)).norm() < 1e-5);
        assert!((rotated.stars[1] - Star::new(-6.0, 4.0, 8.0)).norm() < 1e-5);
    }

    #[test]
    fn test_fov() {
        let fov = FoV::new(1.0, 2.5);
        let proj_stars = fov.project_sky(Sky::from(stars()));
        println!("ps: {:?}", proj_stars);
        assert!((proj_stars[0] - Fpp::new(0.0, 0.2)).norm() < 1e-5);
        assert!((proj_stars[1] - Fpp::new(0.6, 0.32)).norm() < 1e-5);
    }
}
