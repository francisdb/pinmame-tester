use indexmap::IndexMap;

pub type SwitchIndex = IndexMap<u32, &'static str>;

lazy_static! {
// from https://github.com/neophob/wpc-emu


    pub static ref MM_SWITCHES: SwitchIndex = {
        let mut m = IndexMap::new();

        m.insert(1, "COIN#1" );
        m.insert(2, "COIN#2" );
        m.insert(3, "COIN#3" );
        m.insert(4, "COIN#4" );
        m.insert(5, "ESCAPE" );
        m.insert(6, "-" );
        m.insert(7, "+" );
        m.insert(8, "ENTER" );
        m.insert(9, "?" );
        m.insert(10, "MENU / ENTER?" );

        m.insert(11, "LAUNCH BUTTON");
        m.insert(12, "CATAPULT TARGET");
        m.insert(13, "START BUTTON");
        m.insert(14, "PLUMB BOB TILT");
        m.insert(15, "L TROLL TARGET");
        m.insert(16, "LEFT OUTLANE");
        m.insert(17, "RIGHT RETURN");
        m.insert(18, "SHOOTER LANE");

        m.insert(21, "SLAM TILT");
        m.insert(22, "COIN DOOR CLOSED");
        m.insert(25, "R TROLL TARGET");
        m.insert(26, "LEFT RETURN");
        m.insert(27, "RIGHT OUTLANE");
        m.insert(28, "RIGHT EJECT");

        m.insert(31, "TROUGH EJECT");
        m.insert(32, "TROUGH BALL 1");
        m.insert(33, "TROUGH BALL 2");
        m.insert(34, "TROUGH BALL 3");
        m.insert(35, "TROUGH BALL 4");
        m.insert(36, "LEFT POPPER");
        m.insert(37, "CASTLE GATE");
        m.insert(38, "CATAPULT");

        m.insert(41, "MOAT ENTER");
        m.insert(44, "CASTLE LOCK");
        m.insert(45, "L TROLL (U/PLDF)");
        m.insert(46, "R TROLL (U/PLDF)");
        m.insert(47, "LEFT TOP LANE");
        m.insert(48, "RIGHT TOP LANE");

        m.insert(51, "LEFT SLINGSHOT");
        m.insert(52, "RIGHT SLINGSHOT");
        m.insert(53, "LEFT JET");
        m.insert(54, "BOTTOM JET");
        m.insert(55, "RIGHT JET");
        m.insert(56, "DRAWBRIDGE UP");
        m.insert(57, "DRAWBRIDGE DOWN");
        m.insert(58, "TOWER EXIT");

        m.insert(61, "L RAMP ENTER");
        m.insert(62, "L RAMP EXIT");
        m.insert(63, "R RAMP ENTER");
        m.insert(64, "R RAMP EXIT");
        m.insert(65, "LEFT LOOP LO");
        m.insert(66, "LEFT LOOP HI");
        m.insert(67, "RIGHT LOOP LO");
        m.insert(68, "RIGHT LOOP HI");

        m.insert(71, "RIGHT BANK TOP");
        m.insert(72, "RIGHT BANK MID");
        m.insert(73, "RIGHT BANK BOT");
        m.insert(74, "L TROLL UP");
        m.insert(75, "R TROLL UP");

        m
    };

    pub static ref T2_SWITCHES: SwitchIndex = {
        let mut m = IndexMap::new();

        m.insert(1, "COIN#1" );
        m.insert(2, "COIN#2" );
        m.insert(3, "COIN#3" );
        m.insert(4, "?" );
        m.insert(5, "ESCAPE" );
        m.insert(6, "-" );
        m.insert(7, "+" );
        m.insert(8, "ENTER" );
        m.insert(9, "?" );
        m.insert(10, "MENU / ENTER?" );

        m.insert(11, "RIGHT FLIPPER");
        m.insert(12, "LEFT FLIPPER");
        m.insert(13, "START BUTTON");
        m.insert(14, "PLUMB BOB TILT");
        m.insert(15, "TROUGH LEFT");
        m.insert(16, "TROUGH CENTER");
        m.insert(17, "TROUGH RIGHT");
        m.insert(18, "OUTHOLE");

        m.insert(21, "SLAM TILT");
        m.insert(22, "COIN DOOR CLOSED");
        m.insert(23, "TICKED OPTQ");
        m.insert(25, "LEFT OUT LANE");
        m.insert(26, "LEFT RET. LANE");
        m.insert(27, "RIGHT RET. LANE");
        m.insert(28, "RIGHT OUT LANE");

        m.insert(31, "GUN LOADED");
        m.insert(32, "GUN MARK");
        m.insert(33, "GUN HOME");
        m.insert(34, "GRIP TRIGGER");
        m.insert(36, "STAND MID LEFT");
        m.insert(37, "STAND MID CENTER");
        m.insert(38, "STAND MID RIGHT");

        m.insert(41, "LEFT JET");
        m.insert(42, "RIGHT JET");
        m.insert(43, "BOTTOM JET");
        m.insert(44, "LEFT SLING");
        m.insert(45, "RIGHT SLING");
        m.insert(46, "STAND RIGHT TOP");
        m.insert(47, "STAND RIGHT MID");
        m.insert(48, "STAND RIGHT BOT");

        m.insert(51, "LEFT LOCK");
        m.insert(53, "LO ESCAPE ROUTE");
        m.insert(54, "HI ESCAPE ROUTE");
        m.insert(55, "TOP LOCK");
        m.insert(56, "TOP LANE LEFT");
        m.insert(57, "TOP LANE CENTER");
        m.insert(58, "TOP LANE RIGHT");

        m.insert(61, "LEFT RAMP ENTRY");
        m.insert(62, "LEFT RAMP MADE");
        m.insert(63, "RIGHT RAMP ENTRY");
        m.insert(64, "RIGHT RAMP MADE");
        m.insert(65, "LO CHASE LOOP");
        m.insert(66, "HI CHASE LOOP");

        m.insert(71, "TARGET 1 HI");
        m.insert(72, "TARGET 2");
        m.insert(73, "TARGET 3");
        m.insert(74, "TARGET 4");
        m.insert(75, "TARGET 5 LOW");
        m.insert(76, "BALL POPPER");
        m.insert(77, "DROP TARGET");
        m.insert(78, "SHOOTER");

        m
    };

    // pub static ref TABLES: HashMap<&'static str, SwitchIndex> = {
    //     let mut m = HashMap::new();
    //     m.insert("t2", T2_SWITCHES);
    //     m
    // };
}
