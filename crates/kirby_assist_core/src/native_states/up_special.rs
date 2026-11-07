//! Verified kind-specific SPECIAL_HI continuations from skyline-smash d2ca90d.
//! A large custom status number or previous Up-B alone proves nothing.
pub fn up_special_status(kind: i32, status: i32) -> bool {
    if status == 0x1de {
        return true;
    }
    match kind {
        2 => matches!(status, 0x1e2 | 0x1e3),         // LINK
        6 => matches!(status, 0x1fc | 0x1fd | 0x1fe), // KIRBY
        7 => matches!(status, 0x1e1 | 0x1e2 | 0x1e3), // FOX
        8 => matches!(status, 0x1e1 | 0x1e2),         // PIKACHU
        9 => matches!(status, 0x1e6 | 0x1e7 | 0x1e8), // LUIGI
        10 => matches!(status, 0x1e4 | 0x1e5 | 0x1e6 | 0x1e7 | 0x1e8), // NESS
        11 => matches!(status, 0x1e3 | 0x1e4),        // CAPTAIN
        13 => matches!(status, 0x1e4 | 0x1e5 | 0x1e6), // PEACH
        14 => matches!(status, 0x1e4 | 0x1e5 | 0x1e6), // PEACH
        15 => matches!(status, 0x1e1 | 0x1e2),        // KOOPA
        16 => matches!(status, 0x1e7 | 0x1e8),        // SHEIK
        17 => matches!(status, 0x1e3 | 0x1e4),        // ZELDA
        20 => matches!(status, 0x1e2 | 0x1e3 | 0x1e4), // FALCO
        24 => matches!(status, 0x1e6 | 0x1e7),        // GANON
        25 => matches!(status, 0x1e6 | 0x1e7),        // MEWTWO
        26 => matches!(status, 0x1ea | 0x1eb | 0x1ec), // ROY
        27 => matches!(status, 0x1ea | 0x1eb | 0x1ec), // ROY
        28 => matches!(status, 0x1e1 | 0x1e2),        // GAMEWATCH
        29 => matches!(status, 0x1e8),                // METAKNIGHT
        30 => matches!(status, 0x1e7 | 0x1e8),        // PIT
        31 => matches!(status, 0x1e7 | 0x1e8),        // PIT
        33 => matches!(status, 0x1f9),                // WARIO
        34 => matches!(status, 0x1f4 | 0x1f5),        // SNAKE
        35 => matches!(status, 0x1e9 | 0x1ea | 0x1eb), // IKE
        39 => matches!(status, 0x1f0 | 0x1f2 | 0x1f4 | 0x1f5), // DIDDY
        40 => matches!(status, 0x1e4 | 0x1e5 | 0x1e6 | 0x1e7 | 0x1e8), // LUCAS
        41 => matches!(status, 0x1ef),                // SONIC
        42 => matches!(status, 0x1f6 | 0x1f7 | 0x1f8 | 0x1f9 | 0x1fa), // DEDEDE
        43 => matches!(status, 0x1e5 | 0x1e6 | 0x1e7), // PIKMIN
        44 => matches!(status, 0x1e6 | 0x1e7 | 0x1e8), // LUCARIO
        45 => matches!(status, 0x1e3 | 0x1e4 | 0x1e5 | 0x1e6), // ROBOT
        47 => matches!(status, 0x1e1 | 0x1e2 | 0x1e3), // WOLF
        48 => matches!(
            status,
            0x1e9 | 0x1ea | 0x1eb | 0x1ec | 0x1ed | 0x1ee | 0x1ef
        ), // MURABITO
        49 => matches!(status, 0x1e2),                // ROCKMAN
        50 => matches!(status, 0x1e8 | 0x1e9),        // WIIFIT
        51 => matches!(status, 0x1e4 | 0x1e5),        // ROSETTA
        52 => matches!(status, 0x1ec | 0x1ed),        // LITTLEMAC
        53 => matches!(status, 0x1e7 | 0x1e9),        // GEKKOUGA
        54 => matches!(status, 0x1e1 | 0x1e2),        // PALUTENA
        55 => matches!(status, 0x1ec | 0x1ed),        // PACMAN
        56 => matches!(status, 0x1e8 | 0x1e9),        // REFLET
        57 => matches!(status, 0x1e5),                // SHULK
        58 => matches!(status, 0x1e7 | 0x1e8 | 0x1e9 | 0x1ea | 0x1ec), // KOOPAJR
        59 => matches!(status, 0x1e1 | 0x1e2),        // DUCKHUNT
        60 => matches!(status, 0x1f0 | 0x1f1 | 0x1f2 | 0x1f4), // RYU
        61 => matches!(status, 0x1f0 | 0x1f1 | 0x1f2 | 0x1f4), // RYU
        62 => matches!(status, 0x1e3 | 0x1e4 | 0x1e5), // CLOUD
        64 => matches!(status, 0x1ed),                // BAYONETTA
        65 => matches!(status, 0x1eb | 0x1ec | 0x1ed | 0x1ee), // INKLING
        66 => matches!(
            status,
            0x1ee | 0x1ef | 0x1f0 | 0x1f2 | 0x1f4 | 0x1f5 | 0x1f6 | 0x1f7
        ), // RIDLEY
        69 => matches!(status, 0x1ea | 0x1eb | 0x1ec | 0x1ed | 0x1ee), // KROOL
        71 => matches!(status, 0x1e6 | 0x1e7 | 0x1e8 | 0x1e9 | 0x1ea), // GAOGAEN
        72 => matches!(status, 0x1f4 | 0x1f5 | 0x1f6 | 0x1f7 | 0x1f8), // MIIFIGHTER
        73 => matches!(
            status,
            0x1ed | 0x1ee | 0x1ef | 0x1f0 | 0x1f1 | 0x1f2 | 0x1f3 | 0x1f4
        ), // MIISWORDSMAN
        74 => matches!(status, 0x1ef | 0x1f0 | 0x1f1), // MIIGUNNER
        75 => matches!(
            status,
            0x1e2 | 0x1e3 | 0x1e4 | 0x1e5 | 0x1e6 | 0x1e7 | 0x1e8 | 0x1e9
        ), // POPO
        76 => matches!(
            status,
            0x1e2 | 0x1e3 | 0x1e4 | 0x1e5 | 0x1e6 | 0x1e7 | 0x1e8 | 0x1e9
        ), // POPO
        78 => matches!(status, 0x1ee | 0x1ef | 0x1f0 | 0x1f1 | 0x1f2), // MIIENEMYF
        79 => matches!(status, 0x1ee | 0x1ef | 0x1f0 | 0x1f2 | 0x1f4 | 0x1f5), // MIIENEMYS
        80 => matches!(status, 0x1ed | 0x1ee | 0x1ef | 0x1f0), // MIIENEMYG
        81 => matches!(status, 0x1ec | 0x1ed),        // PACKUN
        82 => matches!(status, 0x1ef | 0x1f0 | 0x1f1 | 0x1f2 | 0x1f3 | 0x1f4), // JACK
        83 => matches!(status, 0x1eb | 0x1ec),        // BRAVE
        84 => matches!(status, 0x1f2),                // BUDDY
        85 => matches!(status, 0x1f2 | 0x1f3 | 0x1f4 | 0x1f5), // DOLLY
        86 => matches!(status, 0x1ea | 0x1eb | 0x1ec | 0x1ed), // MASTER
        87 => matches!(status, 0x1f4 | 0x1f5 | 0x1f6 | 0x1f7 | 0x1f8 | 0x1f9), // TANTAN
        88 => matches!(status, 0x20a | 0x20b | 0x20c | 0x20d | 0x20e | 0x20f), // PICKEL
        89 => matches!(status, 0x1ea | 0x1eb | 0x1ec | 0x1ed), // EDGE
        90 => matches!(status, 0x1eb | 0x1ec | 0x1ed), // EFLAME
        91 => matches!(status, 0x1eb | 0x1ec | 0x1ed | 0x1ee), // ELIGHT
        92 => matches!(status, 0x20b | 0x20c),        // DEMON
        _ => false,
    }
}
