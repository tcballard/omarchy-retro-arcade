//! Original fixed layouts. '.' is empty, '#' standard, 'A' armour, 'X' steel.
//! Steel is restricted to isolated columns: no closed pockets or spanning roofs.
pub struct Level {
    pub name: &'static str,
    pub note: &'static str,
    pub rows: &'static [&'static str],
}
pub const LEVELS: [Level; 20] = [
    Level {
        name: "First light",
        note: "Find the paddle's edges to steer rebounds.",
        rows: &["............", "..########..", "..########.."],
    },
    Level {
        name: "Steps",
        note: "Use shallow angles to reach the outer steps.",
        rows: &[
            "#..........#",
            "##........##",
            "###......###",
            "####....####",
        ],
    },
    Level {
        name: "Windows",
        note: "Send a ball through the open windows.",
        rows: &[
            ".###....###.",
            ".#.#....#.#.",
            ".###....###.",
            "............",
            "...######...",
        ],
    },
    Level {
        name: "Rain",
        note: "Track the ball while collecting falling capsules.",
        rows: &[
            "#.#.#.#.#.#.",
            ".#.#.#.#.#.#",
            "#.#.#.#.#.#.",
            ".#.#.#.#.#.#",
        ],
    },
    Level {
        name: "Double take",
        note: "Inset plates take two hits.",
        rows: &["..AAAAAAAA..", "..########..", "..########.."],
    },
    Level {
        name: "Pillars",
        note: "Steel columns reflect without scoring; pass around their ends.",
        rows: &[
            "###..##..###",
            "###X.##.X###",
            "###X.##.X###",
            "...X....X...",
        ],
    },
    Level {
        name: "Crown",
        note: "Climb around the reinforced crown.",
        rows: &[
            "A....AA....A",
            "AA..AAAA..AA",
            "#AAAAAAAAAA#",
            "..########..",
        ],
    },
    Level {
        name: "Gates",
        note: "Aim between isolated steel posts.",
        rows: &[
            ".AAA.AA.AAA.",
            ".###.##.###.",
            "..X..XX..X..",
            "..X......X..",
        ],
    },
    Level {
        name: "Terraces",
        note: "Work through staggered layers.",
        rows: &[
            "AAAA........",
            "####AAAA....",
            "....####AAAA",
            "........####",
            "..##....##..",
        ],
    },
    Level {
        name: "Halfway",
        note: "Armour above steel rewards controlled returns.",
        rows: &[
            ".AAAAAAAAAA.",
            ".##########.",
            "..X......X..",
            "..X.####.X..",
            "....####....",
        ],
    },
    Level {
        name: "Offset",
        note: "Switch between unequal wings.",
        rows: &[
            "AAAAA.......",
            "#####...AAA.",
            "..X.....###.",
            "..X..AA.###.",
            ".....##.....",
        ],
    },
    Level {
        name: "Needle",
        note: "Use the central open channel.",
        rows: &[
            "AAAAA..AAAAA",
            "#####..#####",
            "..X......X..",
            "..X......X..",
            "##........##",
        ],
    },
    Level {
        name: "Stairwell",
        note: "Follow the descending diagonal.",
        rows: &[
            "AAA.........",
            "###AA.......",
            "...##AA.....",
            ".....##AA...",
            ".......##AAA",
            ".........###",
        ],
    },
    Level {
        name: "Islands",
        note: "Keep track of isolated final targets.",
        rows: &[
            "AA...AA...AA",
            "##...##...##",
            "............",
            "...AA..AA...",
            "...##..##...",
            ".X........X.",
        ],
    },
    Level {
        name: "Switchback",
        note: "Clear each wing from below or around the posts.",
        rows: &[
            "AAAA....AAAA",
            "####....####",
            "...X....X...",
            "AA.X....X.AA",
            "##........##",
            "....AAAA....",
        ],
    },
    Level {
        name: "Fortress",
        note: "Break the armour; the steel is never a clear target.",
        rows: &[
            "AAAAAAAAAAAA",
            "##AA####AA##",
            "..X......X..",
            "A.X.AAAA.X.A",
            "#...####...#",
            "..AA....AA..",
        ],
    },
    Level {
        name: "Lattice",
        note: "Thread the gaps and attack from both sides.",
        rows: &[
            "AA.AA.AA.AA.",
            "##.##.##.##.",
            ".X..X..X..X.",
            "AA.AA.AA.AA.",
            "##.##.##.##.",
            ".X..X..X..X.",
        ],
    },
    Level {
        name: "Cascade",
        note: "Long diagonal chains reward multiball.",
        rows: &[
            "AA........AA",
            "#AAA....AAA#",
            ".##AA..AA##.",
            "...AAAAAA...",
            "...######...",
            "AA...XX...AA",
            "##........##",
        ],
    },
    Level {
        name: "Citadel",
        note: "Use the open side lanes to get above the armour.",
        rows: &[
            ".AAAAAAAAAA.",
            ".AA######AA.",
            "..X.AAAA.X..",
            "..X.####.X..",
            "AA........AA",
            "##.AAAAAA.##",
            "...######...",
        ],
    },
    Level {
        name: "Shatter",
        note: "Combine precise rebounds, capsules and patient returns.",
        rows: &[
            "AAAAAAAAAAAA",
            "##AA####AA##",
            ".X..AAAA..X.",
            ".X..####..X.",
            "AA..X..X..AA",
            "##........##",
            "..AAAAAAAA..",
            "...######...",
        ],
    },
];
pub const LEFT: f64 = 28.;
pub const TOP: f64 = 58.;
pub const PITCH_X: f64 = 62.;
pub const PITCH_Y: f64 = 27.;
pub const BRICK_W: f64 = 56.;
pub const BRICK_H: f64 = 20.;
