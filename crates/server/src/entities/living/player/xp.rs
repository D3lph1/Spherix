use bevy_ecs::prelude::Component;

///
/// https://minecraft.fandom.com/wiki/Experience
///
#[derive(Component)]
pub struct Xp(u32);

impl Xp {
    const POINTS_MAX: u32 = u32::pow(2, 31) - 1;

    #[inline]
    pub fn new(points: u32) -> Self {
        if points >= Self::POINTS_MAX {
            Self(Self::POINTS_MAX)
        } else {
            Self(points)
        }
    }

    #[inline]
    pub fn points(&self) -> u32 {
        self.0
    }

    #[inline]
    pub fn level(&self) -> u32 {
        Self::level_from_total_points(self.0)
    }

    #[inline]
    pub fn percentage(&self) -> f32 {
        let p0 = Self::points_to_reach_level(self.level()) as f32;
        let p100 = Self::points_to_reach_level(self.level() + 1) as f32;

        1. - (p100 - self.points() as f32) / (p100 - p0)
    }

    #[inline]
    pub fn set(&mut self, points: u32) {
        self.0 = points;
    }

    #[inline]
    pub fn advance(&mut self, points: u32) {
        self.0 += points;
    }

    pub fn points_req_for_next_level(level: u32) -> u32 {
        match level {
            0..=15 => 2 * level + 7,
            16..=30 => 5 * level - 38,
            31.. => 9 * level - 158
        }
    }

    pub fn points_to_reach_level(level: u32) -> u32 {
        let level_squared = u32::pow(level, 2) as f32;

        (match level {
            0..=16 => level_squared + 6. * level as f32,
            17..=31 => 2.5 * level_squared - 40.5 * level as f32 + 360.,
            32.. => 4.5 * level_squared - 162.5 * level as f32 + 2220.
        }) as u32
    }

    pub fn level_from_total_points(points: u32) -> u32 {
        (match points {
            0..=352 => f32::sqrt(points as f32 + 9.) - 3.,
            353..=1507 => 81. / 10. + f32::sqrt((2. / 5.) * (points as f32 - (7839. / 40.))),
            1508.. => 325. / 18. + f32::sqrt((2. / 9.) * (points as f32 - (54215. / 72.))),
        }) as u32
    }
}

impl Default for Xp {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use spherix_util::assert_f32_eq;

    use crate::entities::living::player::xp::Xp;

    #[test]
    fn points_req_for_next_level() {
        // 0..=15

        assert_eq!(7, Xp::points_req_for_next_level(0));
        assert_eq!(9, Xp::points_req_for_next_level(1));
        assert_eq!(11, Xp::points_req_for_next_level(2));
        assert_eq!(13, Xp::points_req_for_next_level(3));
        assert_eq!(15, Xp::points_req_for_next_level(4));
        assert_eq!(17, Xp::points_req_for_next_level(5));
        assert_eq!(19, Xp::points_req_for_next_level(6));
        assert_eq!(21, Xp::points_req_for_next_level(7));
        assert_eq!(23, Xp::points_req_for_next_level(8));
        assert_eq!(25, Xp::points_req_for_next_level(9));
        assert_eq!(37, Xp::points_req_for_next_level(15));

        // 16..=30

        assert_eq!(42, Xp::points_req_for_next_level(16));
        assert_eq!(47, Xp::points_req_for_next_level(17));
        assert_eq!(52, Xp::points_req_for_next_level(18));
        assert_eq!(57, Xp::points_req_for_next_level(19));
        assert_eq!(77, Xp::points_req_for_next_level(23));
        assert_eq!(82, Xp::points_req_for_next_level(24));
        assert_eq!(87, Xp::points_req_for_next_level(25));
        assert_eq!(92, Xp::points_req_for_next_level(26));
        assert_eq!(97, Xp::points_req_for_next_level(27));
        assert_eq!(112, Xp::points_req_for_next_level(30));

        // 31..

        assert_eq!(121, Xp::points_req_for_next_level(31));
        assert_eq!(130, Xp::points_req_for_next_level(32));
        assert_eq!(139, Xp::points_req_for_next_level(33));
        assert_eq!(148, Xp::points_req_for_next_level(34));
        assert_eq!(157, Xp::points_req_for_next_level(35));
        assert_eq!(166, Xp::points_req_for_next_level(36));
        assert_eq!(175, Xp::points_req_for_next_level(37));
        assert_eq!(184, Xp::points_req_for_next_level(38));
        assert_eq!(193, Xp::points_req_for_next_level(39));
        assert_eq!(202, Xp::points_req_for_next_level(40));
    }

    #[test]
    fn points_to_reach_level() {
        // 0..=16

        assert_eq!(0, Xp::points_to_reach_level(0));
        assert_eq!(7, Xp::points_to_reach_level(1));
        assert_eq!(16, Xp::points_to_reach_level(2));
        assert_eq!(315, Xp::points_to_reach_level(15));

        // 17..=31

        assert_eq!(352, Xp::points_to_reach_level(16));
        assert_eq!(394, Xp::points_to_reach_level(17));
        assert_eq!(441, Xp::points_to_reach_level(18));
        assert_eq!(493, Xp::points_to_reach_level(19));

        assert_eq!(1395, Xp::points_to_reach_level(30));
        assert_eq!(1507, Xp::points_to_reach_level(31));


        // 32..

        assert_eq!(1628, Xp::points_to_reach_level(32));
        assert_eq!(1758, Xp::points_to_reach_level(33));
        assert_eq!(1897, Xp::points_to_reach_level(34));
        assert_eq!(2045, Xp::points_to_reach_level(35));
        assert_eq!(2202, Xp::points_to_reach_level(36));
        assert_eq!(2368, Xp::points_to_reach_level(37));
        assert_eq!(2543, Xp::points_to_reach_level(38));
        assert_eq!(2727, Xp::points_to_reach_level(39));
        assert_eq!(2920, Xp::points_to_reach_level(40));
    }

    #[test]
    fn level_from_total_points() {
        // 0..=352 (At levels 0-16)

        assert_eq!(0, Xp::level_from_total_points(0));
        assert_eq!(0, Xp::level_from_total_points(1));
        assert_eq!(0, Xp::level_from_total_points(2));
        assert_eq!(0, Xp::level_from_total_points(6));

        assert_eq!(1, Xp::level_from_total_points(7));
        assert_eq!(1, Xp::level_from_total_points(8));
        assert_eq!(1, Xp::level_from_total_points(15));

        assert_eq!(2, Xp::level_from_total_points(16));
        assert_eq!(2, Xp::level_from_total_points(20));
        assert_eq!(2, Xp::level_from_total_points(26));

        assert_eq!(15, Xp::level_from_total_points(351));
        assert_eq!(16, Xp::level_from_total_points(352));

        // 353..=1507 (At levels 17–31)

        assert_eq!(16, Xp::level_from_total_points(353));
        assert_eq!(16, Xp::level_from_total_points(390));
        assert_eq!(17, Xp::level_from_total_points(400));
        assert_eq!(17, Xp::level_from_total_points(440));

        assert_eq!(18, Xp::level_from_total_points(441));
        assert_eq!(18, Xp::level_from_total_points(442));
        assert_eq!(18, Xp::level_from_total_points(492));

        assert_eq!(19, Xp::level_from_total_points(493));
        assert_eq!(19, Xp::level_from_total_points(500));
        assert_eq!(19, Xp::level_from_total_points(549));

        assert_eq!(30, Xp::level_from_total_points(1506));
        assert_eq!(31, Xp::level_from_total_points(1507));

        // 1508.. (At levels 32+)

        assert_eq!(31, Xp::level_from_total_points(1508));
        assert_eq!(31, Xp::level_from_total_points(1520));
        assert_eq!(32, Xp::level_from_total_points(1628));
        assert_eq!(32, Xp::level_from_total_points(1629));

        assert_eq!(33, Xp::level_from_total_points(1758));
        assert_eq!(33, Xp::level_from_total_points(1759));
        assert_eq!(33, Xp::level_from_total_points(1896));

        assert_eq!(34, Xp::level_from_total_points(1897));
        assert_eq!(34, Xp::level_from_total_points(2043));
        assert_eq!(34, Xp::level_from_total_points(2044));

        assert_eq!(35, Xp::level_from_total_points(2045));
        assert_eq!(35, Xp::level_from_total_points(2060));
    }

    #[test]
    fn complex() {
        let xp = Xp::new(18);
        assert_eq!(2, xp.level());
        assert_f32_eq!(0.182, xp.percentage(), 3);
    }
}
