//! Pinned kind-specific Final Smash phases; do not apply these numbers to
//! another fighter where they may mean ordinary attacks or recovery.
pub(super) fn final_status(kind: i32, status: i32) -> bool {
    match kind {
        1 => matches!(status, 0x200 | 0x201), // DONKEY
        2 => matches!(status, 0x1e6 | 0x1e7), // LINK
        3 => matches!(
            status,
            0x1ef | 0x1f0 | 0x1f1 | 0x1f2 | 0x1f3 | 0x1f4 | 0x1f5
        ), // SAMUS
        5 => matches!(status, 0x1ea | 0x1eb | 0x1ec | 0x1ed | 0x1ee), // YOSHI
        6 => matches!(status, 0x201 | 0x202 | 0x203 | 0x204 | 0x205), // KIRBY
        7 => matches!(status, 0x1e7 | 0x1e8 | 0x1e9), // FOX
        8 => matches!(
            status,
            0x1e8 | 0x1e9 | 0x1ea | 0x1eb | 0x1ec | 0x1ed | 0x1ee | 0x1ef
        ), // PIKACHU
        9 => matches!(status, 0x1e9 | 0x1ea | 0x1eb | 0x1ec | 0x1ed), // LUIGI
        10 => matches!(status, 0x1ec),        // NESS
        11 => matches!(status, 0x1e7 | 0x1e8 | 0x1e9 | 0x1ea | 0x1eb), // CAPTAIN
        12 => matches!(status, 0x1e8 | 0x1e9), // PURIN
        15 => matches!(status, 0x1ea),        // KOOPA
        16 => matches!(
            status,
            0x1ec | 0x1ed | 0x1ee | 0x1ef | 0x1f0 | 0x1f1 | 0x1f2
        ), // SHEIK
        17 => matches!(status, 0x1e7 | 0x1e8 | 0x1e9 | 0x1ea), // ZELDA
        20 => matches!(status, 0x1e5 | 0x1e6 | 0x1e7), // FALCO
        21 => matches!(status, 0x1e8 | 0x1e9 | 0x1ea), // MARTH
        24 => matches!(status, 0x1ea | 0x1eb), // GANON
        25 => matches!(status, 0x1e8),        // MEWTWO
        26 => matches!(status, 0x1ee | 0x1ef | 0x1f0 | 0x1f1), // ROY
        27 => matches!(status, 0x1ee | 0x1ef | 0x1f0 | 0x1f1), // ROY
        28 => matches!(
            status,
            0x1e9
                | 0x1ea
                | 0x1eb
                | 0x1ec
                | 0x1ed
                | 0x1ee
                | 0x1ef
                | 0x1f0
                | 0x1f1
                | 0x1f2
                | 0x1f3
                | 0x1f4
                | 0x1f5
                | 0x1f6
                | 0x1f7
                | 0x1f8
                | 0x1f9
                | 0x1fa
                | 0x1fb
                | 0x1fc
                | 0x1fd
                | 0x1fe
                | 0x1ff
                | 0x200
                | 0x201
                | 0x202
                | 0x203
                | 0x204
        ), // GAMEWATCH
        29 => matches!(status, 0x1e9 | 0x1ea | 0x1eb | 0x1ec | 0x1ed), // METAKNIGHT
        30 => matches!(status, 0x1eb | 0x1ec | 0x1ed), // PIT
        31 => matches!(status, 0x1eb | 0x1ec | 0x1ed), // PIT
        32 => matches!(status, 0x1ea | 0x1eb), // SZEROSUIT
        33 => matches!(status, 0x1fb | 0x1fc | 0x1fd | 0x1fe | 0x1ff), // WARIO
        34 => matches!(status, 0x1fd | 0x1fe | 0x1ff | 0x200), // SNAKE
        35 => matches!(status, 0x1ed | 0x1ee | 0x1ef | 0x1f0 | 0x1f1), // IKE
        39 => matches!(
            status,
            0x1f7 | 0x1f8 | 0x1f9 | 0x1fa | 0x1fb | 0x1fc | 0x1fd | 0x1fe
        ), // DIDDY
        40 => matches!(status, 0x1ec),        // LUCAS
        41 => matches!(status, 0x1f2),        // SONIC
        42 => matches!(status, 0x204 | 0x205 | 0x206 | 0x207 | 0x208), // DEDEDE
        43 => matches!(status, 0x1e1 | 0x1e2 | 0x1e3 | 0x1e4), // PIKMIN
        44 => matches!(status, 0x1ec | 0x1ed | 0x1ee | 0x1ef), // LUCARIO
        45 => matches!(status, 0x1e9 | 0x1ea), // ROBOT
        47 => matches!(status, 0x1e7 | 0x1e8 | 0x1e9), // WOLF
        48 => matches!(status, 0x1ff | 0x200 | 0x201 | 0x202 | 0x203), // MURABITO
        49 => matches!(status, 0x1eb | 0x1ec | 0x1ed | 0x1ee | 0x1ef), // ROCKMAN
        50 => matches!(status, 0x1ed | 0x1ee), // WIIFIT
        52 => matches!(status, 0x1ef | 0x1f0 | 0x1f1 | 0x1f2 | 0x1f3), // LITTLEMAC
        53 => matches!(status, 0x1ef | 0x1f0), // GEKKOUGA
        54 => matches!(status, 0x1e5 | 0x1e6 | 0x1e7 | 0x1e8), // PALUTENA
        55 => matches!(status, 0x1ee | 0x1ef), // PACMAN
        56 => matches!(
            status,
            0x1ec
                | 0x1ed
                | 0x1ee
                | 0x1ef
                | 0x1f0
                | 0x1f1
                | 0x1f2
                | 0x1f3
                | 0x1f4
                | 0x1f5
                | 0x1f6
                | 0x1f7
                | 0x1fa
                | 0x1fb
                | 0x1fc
        ), // REFLET
        57 => matches!(status, 0x1e9 | 0x1ea | 0x1eb | 0x1ec), // SHULK
        58 => matches!(status, 0x1ed | 0x1ee), // KOOPAJR
        59 => matches!(
            status,
            0x1e4 | 0x1e5 | 0x1e6 | 0x1e7 | 0x1e8 | 0x1e9 | 0x1ea
        ), // DUCKHUNT
        60 => matches!(
            status,
            0x1f9 | 0x1fa | 0x1fb | 0x1fc | 0x1fd | 0x1fe | 0x1ff | 0x200 | 0x201
        ), // RYU
        61 => matches!(
            status,
            0x1f9 | 0x1fa | 0x1fb | 0x1fc | 0x1fd | 0x1fe | 0x1ff | 0x200 | 0x201
        ), // RYU
        62 => matches!(
            status,
            0x1e8
                | 0x1e9
                | 0x1ea
                | 0x1eb
                | 0x1ec
                | 0x1ed
                | 0x1ee
                | 0x1ef
                | 0x1f0
                | 0x1f1
                | 0x1f2
                | 0x1f3
                | 0x1f4
                | 0x1f5
                | 0x1f6
        ), // CLOUD
        63 => matches!(status, 0x1ef | 0x1f0 | 0x1f1 | 0x1f2 | 0x1f3), // KAMUI
        64 => matches!(status, 0x1ee | 0x1ef | 0x1f0 | 0x1f1), // BAYONETTA
        66 => matches!(
            status,
            0x1f9 | 0x1fa | 0x1fb | 0x1fc | 0x1fd | 0x1fe | 0x1ff | 0x200 | 0x201
        ), // RIDLEY
        67 => matches!(status, 0x1e7 | 0x1e8 | 0x1e9 | 0x1ea | 0x1eb), // SIMON
        69 => matches!(
            status,
            0x1f1 | 0x1f2 | 0x1f3 | 0x1f4 | 0x1f5 | 0x1f6 | 0x1f7 | 0x1f8
        ), // KROOL
        71 => matches!(
            status,
            0x1ed | 0x1ee | 0x1ef | 0x1f0 | 0x1f1 | 0x1f2 | 0x1f3
        ), // GAOGAEN
        72 => matches!(status, 0x203 | 0x204 | 0x205 | 0x206), // MIIFIGHTER
        73 => matches!(status, 0x1f7 | 0x1f8), // MIISWORDSMAN
        74 => matches!(status, 0x1f8 | 0x1f9), // MIIGUNNER
        75 => matches!(status, 0x1ed | 0x1ee), // POPO
        76 => matches!(status, 0x1ed | 0x1ee), // POPO
        79 => matches!(status, 0x1f8 | 0x1f9), // MIIENEMYS
        82 => matches!(status, 0x1f6 | 0x1f7 | 0x1f8 | 0x1f9 | 0x1fa | 0x1fb), // JACK
        83 => matches!(status, 0x1fb | 0x1fc | 0x1fd), // BRAVE
        84 => matches!(status, 0x1f5 | 0x1f6 | 0x1f7 | 0x1f8 | 0x1f9 | 0x1fa), // BUDDY
        85 => matches!(
            status,
            0x1fc | 0x1fd | 0x1fe | 0x1ff | 0x200 | 0x201 | 0x202
        ), // DOLLY
        86 => matches!(status, 0x1f3 | 0x1f4 | 0x1f5 | 0x1f6 | 0x1f7), // MASTER
        87 => matches!(status, 0x1fa | 0x1fb | 0x1fc | 0x1fd | 0x1fe), // TANTAN
        88 => matches!(status, 0x218 | 0x219 | 0x21a), // PICKEL
        89 => matches!(status, 0x1f0 | 0x1f1 | 0x1f2), // EDGE
        90 => matches!(status, 0x1e3 | 0x1e4 | 0x1e5 | 0x1e6), // EFLAME
        91 => matches!(status, 0x1e3 | 0x1e4 | 0x1e5 | 0x1e6 | 0x1ef | 0x1f0), // ELIGHT
        92 => matches!(status, 0x210 | 0x211 | 0x212), // DEMON
        93 => matches!(status, 0x1ef | 0x1f0 | 0x1f1 | 0x1f2 | 0x1f3 | 0x1f4), // TRAIL
        _ => false,
    }
}
