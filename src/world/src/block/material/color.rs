#[derive(PartialEq, Eq)]
pub struct Color {
    id: u8
}

impl Color {
    pub const NONE: Color = Color::new(0);
    pub const GRASS: Color = Color::new(1);
    pub const SAND: Color = Color::new(2);
    pub const WOOL: Color = Color::new(3);
    pub const FIRE: Color = Color::new(4);
    pub const ICE: Color = Color::new(5);
    pub const METAL: Color = Color::new(6);
    pub const PLANT: Color = Color::new(7);
    pub const SNOW: Color = Color::new(8);
    pub const CLAY: Color = Color::new(9);
    pub const DIRT: Color = Color::new(10);
    pub const STONE: Color = Color::new(11);
    pub const WATER: Color = Color::new(12);
    pub const WOOD: Color = Color::new(13);
    pub const QUARTZ: Color = Color::new(14);
    pub const COLOR_ORANGE: Color = Color::new(15);
    pub const COLOR_MAGENTA: Color = Color::new(16);
    pub const COLOR_LIGHT_BLUE: Color = Color::new(17);
    pub const COLOR_YELLOW: Color = Color::new(18);
    pub const COLOR_LIGHT_GREEN: Color = Color::new(19);
    pub const COLOR_PINK: Color = Color::new(20);
    pub const COLOR_GRAY: Color = Color::new(21);
    pub const COLOR_LIGHT_GRAY: Color = Color::new(22);
    pub const COLOR_CYAN: Color = Color::new(23);
    pub const COLOR_PURPLE: Color = Color::new(24);
    pub const COLOR_BLUE: Color = Color::new(25);
    pub const COLOR_BROWN: Color = Color::new(26);
    pub const COLOR_GREEN: Color = Color::new(27);
    pub const COLOR_RED: Color = Color::new(28);
    pub const COLOR_BLACK: Color = Color::new(29);
    pub const GOLD: Color = Color::new(30);
    pub const DIAMOND: Color = Color::new(31);
    pub const LAPIS: Color = Color::new(32);
    pub const EMERALD: Color = Color::new(33);
    pub const PODZOL: Color = Color::new(34);
    pub const NETHER: Color = Color::new(35);
    pub const TERRACOTTA_WHITE: Color = Color::new(36);
    pub const TERRACOTTA_ORANGE: Color = Color::new(37);
    pub const TERRACOTTA_MAGENTA: Color = Color::new(38);
    pub const TERRACOTTA_LIGHT_BLUE: Color = Color::new(39);
    pub const TERRACOTTA_YELLOW: Color = Color::new(40);
    pub const TERRACOTTA_LIGHT_GREEN: Color = Color::new(41);
    pub const TERRACOTTA_PINK: Color = Color::new(42);
    pub const TERRACOTTA_GRAY: Color = Color::new(43);
    pub const TERRACOTTA_LIGHT_GRAY: Color = Color::new(44);
    pub const TERRACOTTA_CYAN: Color = Color::new(45);
    pub const TERRACOTTA_PURPLE: Color = Color::new(46);
    pub const TERRACOTTA_BLUE: Color = Color::new(47);
    pub const TERRACOTTA_BROWN: Color = Color::new(48);
    pub const TERRACOTTA_GREEN: Color = Color::new(49);
    pub const TERRACOTTA_RED: Color = Color::new(50);
    pub const TERRACOTTA_BLACK: Color = Color::new(51);
    pub const CRIMSON_NYLIUM: Color = Color::new(52);
    pub const CRIMSON_STEM: Color = Color::new(53);
    pub const CRIMSON_HYPHAE: Color = Color::new(54);
    pub const WARPED_NYLIUM: Color = Color::new(55);
    pub const WARPED_STEM: Color = Color::new(56);
    pub const WARPED_HYPHAE: Color = Color::new(57);
    pub const WARPED_WART_BLOCK: Color = Color::new(58);
    pub const DEEPSLATE: Color = Color::new(59);
    pub const RAW_IRON: Color = Color::new(60);
    pub const GLOW_LICHEN: Color = Color::new(61);

    pub const fn new(id: u8) -> Self {
        Self {id}
    }
}
