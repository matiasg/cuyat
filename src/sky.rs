use itertools::Itertools;
use std::{collections::HashMap, f32::consts::PI, fs};

use nalgebra::{DVector, Dyn, OMatrix, OVector, SVector, UnitQuaternion, U3};
use regex::Regex;

type SkyMat = OMatrix<f32, Dyn, U3>;
pub type Star = SVector<f32, 3>;
type Position = SVector<f32, 3>;
pub type Fpp = SVector<f32, 2>; // Focal Plane Point
pub type FPStars = Vec<(Fpp, Brightness, String)>;

/// Star (position), Brightness, Name
pub type StBrNm = (Star, Brightness, String);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Brightness {
    brightness: f32, // expected to be between 0 and 1
}
impl Brightness {
    const MAX_MAG: f32 = -1.46f32;

    fn for_magnitude(m: f32) -> Self {
        let brightness: f32 = 0.01f32.powf((m - Self::MAX_MAG) / 5.0);
        Self { brightness }
    }
    fn new(b: f32) -> Self {
        Self { brightness: b }
    }
}

#[derive(Clone, Debug)]
pub struct Sky {
    stars: Vec<StBrNm>,
}

impl Sky {
    pub fn new(catalog: &Option<String>, nstars: usize) -> Self {
        match catalog {
            None => Self::random_with_stars(nstars),
            Some(ref filename) => {
                Self::from_converted_file(filename.as_str(), nstars).with_random_quaternion()
            }
        }
    }
    pub fn from(stars: &[StBrNm]) -> Self {
        Self {
            stars: stars.to_vec(),
        }
    }

    pub fn from_line(line: &str, sbn_re: &Regex) -> StBrNm {
        let sbn = sbn_re.captures(line).unwrap();

        let name = String::from(sbn.get(1).unwrap().as_str());

        let rahh: u8 = sbn.get(2).unwrap().as_str().parse().unwrap();
        let ramm: u8 = sbn.get(3).unwrap().as_str().parse().unwrap();
        let rass: f32 = sbn.get(4).unwrap().as_str().parse().unwrap();
        let ra: f32 = ((rahh as f32) * 15.0 + (ramm as f32) / 4.0 + rass / 240.0).to_radians();

        let sgn: f32 = match sbn.get(5).unwrap().as_str() {
            "+" => 1.0,
            _ => -1.0,
        };
        let dedeg: u8 = sbn.get(6).unwrap().as_str().parse().unwrap();
        let demin: u8 = sbn.get(7).unwrap().as_str().parse().unwrap();
        let desec: u8 = sbn.get(8).unwrap().as_str().parse().unwrap();
        let dec: f32 =
            sgn * ((dedeg as f32) + (demin as f32) / 60.0 + (desec as f32) / 3600.0).to_radians();

        let star_pos = Star::new(ra.cos() * dec.cos(), ra.sin() * dec.cos(), dec.sin());

        let sgn: f32 = match sbn.get(9).unwrap().as_str() {
            "-" => -1.0,
            _ => 1.0,
        };
        let mag: f32 = sbn.get(10).unwrap().as_str().trim().parse().unwrap();
        let brightness = Brightness::for_magnitude(sgn * mag);
        (star_pos, brightness, name)
    }

    pub fn from_catalog_file(fname: &str) -> Self {
        let sbn_re = Regex::new("^.{7}(.{7}).{61}(\\d\\d)(\\d\\d)(\\d\\d\\.\\d)([+-])(\\d\\d)(\\d\\d)(\\d\\d).{12}([+ -])([0-9. ]{4})").unwrap();
        let input: String = fs::read_to_string(fname).unwrap();
        let input: Vec<&str> = input.trim_end().split('\n').collect();
        let stars: Vec<StBrNm> = input
            .iter()
            .map(|&line| Self::from_line(line, &sbn_re))
            .filter(|sbn| sbn.1.brightness > 0.01)
            .collect();
        Self::from(&stars)
    }

    pub fn from_converted_file(fname: &str, nstars: usize) -> Self {
        let sbn_re = Regex::new("^(.{5}),(\\d\\d)(\\d\\d)(\\d\\d\\.\\d),([+-])(\\d\\d)(\\d\\d)(\\d\\d),(-?)([0-9. ]{4})").unwrap();
        let input: String = fs::read_to_string(fname).unwrap();
        let input: Vec<&str> = input.trim_end().split('\n').collect();
        let mut stars: Vec<StBrNm> = input
            .iter()
            .map(|&line| Self::from_line(line, &sbn_re))
            .filter(|sbn| sbn.1.brightness > 0.01)
            .collect();
        stars.sort_by(|sbn1, sbn2| sbn1.1.brightness.total_cmp(&sbn2.1.brightness));
        let eff_nstars = stars.len().min(nstars);
        Self::from(stars.get(stars.len() - eff_nstars..).unwrap())
    }
    pub fn convert_catalog_file(
        infile: &str,
        outfile: &str,
        max_magnitude: f32,
    ) -> Result<u8, std::io::Error> {
        let sbn_re = Regex::new("^.{7}(.{7}).{61}(\\d\\d\\d\\d\\d\\d\\.\\d)([+-]\\d\\d\\d\\d\\d\\d).{12}([+ -][0-9. ]{4})").unwrap();
        let conversion_map = greek_names_map();
        let input: String = fs::read_to_string(infile).unwrap();
        let input: Vec<&str> = input.trim_end().split('\n').collect();
        let outb: Vec<String> = input
            .iter()
            .filter_map(|line| {
                let sbn = sbn_re.captures(line).unwrap();
                let name = String::from(sbn.get(1).unwrap().as_str());
                let name = format!(
                    "{}{}",
                    conversion_map[name.get(0..3).unwrap()],
                    name.get(3..).unwrap()
                );
                let ra = String::from(sbn.get(2).unwrap().as_str());
                let dec = String::from(sbn.get(3).unwrap().as_str());
                let mag: f32 = sbn.get(4).unwrap().as_str().trim().parse().unwrap();
                if mag <= max_magnitude {
                    Some(format!("{name},{ra},{dec},{mag:.2}"))
                } else {
                    None
                }
            })
            .collect();

        fs::write(outfile, outb.join("\n"))?;
        Ok(0)
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
            stars: self
                .stars
                .iter()
                .map(|(s, b, n)| (*s - pos, *b, n.clone()))
                .collect(),
        }
    }

    pub fn with_attitude(&self, q: UnitQuaternion<f32>) -> Self {
        Self {
            stars: self
                .stars
                .iter()
                .map(|(s, b, n)| (q * *s, *b, n.clone()))
                .collect(),
        }
    }

    pub fn random_with_stars(n: usize) -> Self {
        let stars_positions: Vec<Star> = (0..n).map(|_| Star::new_random() * 10.0).collect();
        // FIXME: use better probability density of brightnesses
        let brightnesses: DVector<f32> = DVector::<f32>::new_random(n);
        let prefs: Vec<&str> = greek_names_map().values().copied().collect();
        let consts: Vec<char> = ('a'..='z').chain('A'..='Z').collect();
        let names = consts
            .iter()
            .cartesian_product(prefs.iter())
            .map(|(c, p)| format!("{p}{c}"));

        // let names: Vec<String> = (0..n).map(|i| format!("{i}")).collect();
        let stars: Vec<StBrNm> = stars_positions
            .iter()
            .copied()
            .zip(brightnesses.iter().map(|&b| Brightness::new(b)))
            .zip(names)
            .map(|((s, b), n)| (s, b, n))
            .collect();
        let sky = Self { stars };
        sky.seen_from(Star::new(5.0, 5.0, 5.0))
    }

    pub fn with_random_quaternion(&self) -> Sky {
        self.with_attitude(random_quaternion())
    }
}

fn greek_names_map<'a>() -> HashMap<&'a str, &'a str> {
    HashMap::from([
        ("   ", " "),
        ("Alp", "α"),
        ("Bet", "β"),
        ("Gam", "γ"),
        ("Del", "δ"),
        ("Eps", "ε"),
        ("Zet", "ζ"),
        ("Eta", "η"),
        ("The", "θ"),
        ("Iot", "ι"),
        ("Kap", "κ"),
        ("Lam", "λ"),
        ("Mu ", "μ"),
        ("Nu ", "ν"),
        ("Xi ", "ξ"),
        ("Omi", "ο"),
        ("Pi ", "π"),
        ("Rho", "ρ"),
        ("Sig", "σ"),
        ("Tau", "τ"),
        ("Psi", "ψ"),
        ("Phi", "φ"),
        ("Ups", "υ"),
        ("Ome", "ω"),
        ("Chi", "χ"),
    ])
}

pub fn random_quaternion() -> nalgebra::Unit<nalgebra::Quaternion<f32>> {
    let rpy: OVector<f32, U3> = OVector::<f32, U3>::new_random() * 2.0 * PI;
    UnitQuaternion::from_euler_angles(rpy[0], rpy[1], rpy[2])
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
    pub fn rescale(&self, scale: f32) -> Self {
        Self {
            half_fov_x: self.half_fov_x * scale,
            half_fov_y: self.half_fov_y * scale,
        }
    }
    pub fn zoom(&self) -> f32 {
        self.half_fov_x
    }
    fn can_be_seen(&self, b: &Brightness) -> bool {
        b.brightness / self.half_fov_x > 0.01f32.powf(0.8)
    }
    pub fn project(&self, star: &Star) -> Fpp {
        Fpp::new(
            star[0] / star[2] / self.half_fov_x,
            star[1] / star[2] / self.half_fov_y,
        )
    }
    pub fn project_sky(&self, sky: &Sky) -> FPStars {
        sky.stars
            .iter()
            .map(|(s, b, n)| (self.project(s), *b, n.clone()))
            .collect()
    }
    fn in_box(x: f32, y: f32, maxx: u8, maxy: u8) -> Option<(u8, u8)> {
        if x < 0.0 || x >= maxx as f32 || y < 0.0 || y >= maxy as f32 {
            None
        } else {
            Some((x as u8, y as u8))
        }
    }
    pub fn to_screen(&self, star: &Star, maxx: u8, maxy: u8) -> Option<(u8, u8)> {
        if star[2] <= 0.0 {
            return None;
        }
        let fpp = self.project(star);
        let x = ((fpp[0] + 1.0) / 2.0 * (maxx as f32)).round();
        let y = ((fpp[1] + 1.0) / 2.0 * (maxy as f32)).round();
        Self::in_box(x, y, maxx, maxy)
    }
    pub fn project_sky_to_screen(
        &self,
        sky: Sky,
        maxx: u8,
        maxy: u8,
    ) -> Vec<Option<(u8, u8, u8, String)>> {
        sky.stars
            .iter()
            .map(|(s, b, n)| {
                let sp = self.to_screen(s, maxx, maxy);
                if sp.is_none() || !self.can_be_seen(b) {
                    None
                } else {
                    let sp = sp.unwrap();
                    let bu = 128 + (b.brightness * 127.0).floor() as u8;
                    Some((sp.0, sp.1, bu, String::from(n)))
                }
            })
            .collect()
    }

    pub fn with_angles(x_rad: f32, y_rad: f32) -> Self {
        Self {
            half_fov_x: x_rad.tan() / 2.0,
            half_fov_y: y_rad.tan() / 2.0,
        }
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use regex::Regex;
    use std::f32::consts::PI;

    use nalgebra::UnitQuaternion;

    use super::{Brightness, FoV, Fpp, Position, Sky, StBrNm, Star};

    fn stars() -> Vec<StBrNm> {
        vec![
            (
                Star::new(0.0, 1.0, 2.0),
                Brightness::new(0.5),
                String::from("a"),
            ),
            (
                Star::new(3.0, 4.0, 5.0),
                Brightness::new(0.25),
                String::from("b"),
            ),
        ]
    }
    #[test]
    fn test_sky() {
        let sky = Sky::from(&stars());
        assert_eq!(sky.len(), 2);
        let pos = Position::new(-1.0, -2.0, -3.0);
        let from_pos = sky.seen_from(pos);
        assert_eq!(from_pos.len(), 2);
        assert_eq!(
            from_pos.stars,
            vec![
                (
                    Star::new(1.0, 3.0, 5.0),
                    Brightness::new(0.5),
                    String::from("a")
                ),
                (
                    Star::new(4.0, 6.0, 8.0),
                    Brightness::new(0.25),
                    String::from("b")
                )
            ]
        );
        let q = UnitQuaternion::from_euler_angles(0.0, 0.0, PI / 2.0);
        let rotated = from_pos.with_attitude(q);
        assert_eq!(rotated.len(), 2);
        assert!((rotated.stars[0].0 - Star::new(-3.0, 1.0, 5.0)).norm() < 1e-5);
        assert!((rotated.stars[1].0 - Star::new(-6.0, 4.0, 8.0)).norm() < 1e-5);
    }

    #[test]
    fn test_fov() {
        let fov = FoV::new(1.0, 2.5);
        let proj_stars = fov.project_sky(&Sky::from(&stars()));
        assert!((proj_stars[0].0 - Fpp::new(0.0, 0.2)).norm() < 1e-5);
        assert!((proj_stars[1].0 - Fpp::new(0.6, 0.32)).norm() < 1e-5);
    }

    #[test]
    fn test_project() {
        let sky = Sky::from(&stars());
        let fov = FoV::new(1.0, 1.0);
        let p: Vec<_> = fov
            .project_sky_to_screen(sky.clone(), 60, 60)
            .into_iter()
            .flatten()
            .collect();
        assert_eq!(p.len(), 2);
        let (a, b) = (p.get(0).unwrap(), p.get(1).unwrap());
        assert_eq!((a.0, a.1), (30, 45));
        assert_eq!((b.0, b.1), (48, 54));

        let p: Vec<_> = FoV::new(0.5, 0.51)
            .project_sky_to_screen(sky.clone(), 60, 60)
            .into_iter()
            .flatten()
            .collect();
        assert_eq!(p.len(), 1);
        let a = p.first().unwrap();
        assert_eq!((a.0, a.1), (30, 59));

        let p: Vec<_> = FoV::new(0.5, 0.5)
            .project_sky_to_screen(sky.clone(), 60, 60)
            .into_iter()
            .flatten()
            .collect();
        assert_eq!(p.len(), 0);

        let p: Vec<_> = FoV::new(0.5, 0.5)
            .project_sky_to_screen(
                sky.with_attitude(UnitQuaternion::from_euler_angles(0.0, 0.0, PI)),
                60,
                60,
            )
            .into_iter()
            .flatten()
            .collect();
        assert_eq!(p.len(), 1);
        let a = p.first().unwrap();
        assert_eq!((a.0, a.1), (30, 0));
    }

    #[test]
    fn test_from_line() {
        let sbn_re = Regex::new("^.{7}(.{7}).{61}(\\d\\d)(\\d\\d)(\\d\\d\\.\\d)([+-])(\\d\\d)(\\d\\d)(\\d\\d).{12}([+ -])([0-9. ]{4})").unwrap();
        let sbn_re_conv = Regex::new("^(.{5}),(\\d\\d)(\\d\\d)(\\d\\d\\.\\d),([+-])(\\d\\d)(\\d\\d)(\\d\\d),(-?)([0-9. ]{4})").unwrap();

        let bet_line = "2061 58Alp OriBD+07 1055  39801113271 224I   4506  Alp Ori  054945.4+072319055510.3+072425199.79-08.96 0.50  +1.85 +2.06 +1.28   M1-2Ia-Iab        e+0.026+0.009 +.005+021SB         9.9 174.4AE   6*";
        let bet_line_conv = "α Ori,055510.3,+072425,0.50";

        let sir_line = "2491  9Alp CMaBD-16 1591  48915151881 257I   5423           064044.6-163444064508.9-164258227.22-08.88-1.46   0.00 -0.05 -0.03   A1Vm               -0.553-1.205 +.375-008SBO    13 10.3  11.2AB   4*";
        let sir_line_conv = "α CMa,064508.9,-164258,-1.46";

        let betelgeuse = Sky::from_line(bet_line, &sbn_re);
        let bet_conv = Sky::from_line(bet_line_conv, &sbn_re_conv);
        let exp_bet = Star::new(0.0208902, 0.9914355, 0.1289158);
        (0..3)
            .for_each(|i| assert_relative_eq!(betelgeuse.0[i], exp_bet[i], epsilon = f32::EPSILON));
        assert_eq!(betelgeuse.1, Brightness::for_magnitude(0.5));
        assert_eq!(betelgeuse.2, "Alp Ori");
        assert_eq!(bet_conv.0, betelgeuse.0);
        assert_eq!(bet_conv.1, betelgeuse.1);

        let sirius = Sky::from_line(sir_line, &sbn_re);
        let sir_conv = Sky::from_line(sir_line_conv, &sbn_re_conv);
        let exp_sir = Star::new(-0.18745413, 0.93921775, -0.2876299);

        (0..3).for_each(|i| assert_relative_eq!(sirius.0[i], exp_sir[i], epsilon = f32::EPSILON));
        assert_eq!(sirius.1, Brightness::for_magnitude(-1.46));
        assert_eq!(sirius.2, "Alp CMa");
        assert_eq!(sir_conv.0, sirius.0);
        assert_eq!(sir_conv.1, sirius.1);
    }
}
