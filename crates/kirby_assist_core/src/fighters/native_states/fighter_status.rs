//! Fighter-specific status IDs verified against the pinned native constant table.
//! The fighter prefix matters because the same number can mean different states.

/// Bayonetta's Final Smash ending status.
pub const BAYONETTA_FINAL_END: i32 = 0x1f0;
/// Bayonetta's Final Smash post-ending phase status.
pub const BAYONETTA_FINAL_END_AFTER: i32 = 0x1f1;
/// Bayonetta's Final Smash cinematic phase 1 status.
pub const BAYONETTA_FINAL_SCENE01: i32 = 0x1ef;
/// Bayonetta's Final Smash startup status.
pub const BAYONETTA_FINAL_START: i32 = 0x1ee;
/// Bayonetta's up special jump phase status.
pub const BAYONETTA_SPECIAL_HI_JUMP: i32 = 0x1ed;
/// Hero's Final Smash ending status.
pub const BRAVE_FINAL_END: i32 = 0x1fd;
/// Hero's Final Smash ready phase status.
pub const BRAVE_FINAL_READY: i32 = 0x1fb;
/// Hero's Final Smash cinematic phase 1 status.
pub const BRAVE_FINAL_SCENE01: i32 = 0x1fc;
/// Hero's up special charge hold status.
pub const BRAVE_SPECIAL_HI_HOLD: i32 = 0x1eb;
/// Hero's up special jump phase status.
pub const BRAVE_SPECIAL_HI_JUMP: i32 = 0x1ec;
/// Banjo & Kazooie's Final Smash ending status.
pub const BUDDY_FINAL_END: i32 = 0x1fa;
/// Banjo & Kazooie's Final Smash ready phase status.
pub const BUDDY_FINAL_READY: i32 = 0x1f5;
/// Banjo & Kazooie's Final Smash cinematic phase 1 status.
pub const BUDDY_FINAL_SCENE01: i32 = 0x1f6;
/// Banjo & Kazooie's Final Smash cinematic phase 2 status.
pub const BUDDY_FINAL_SCENE02: i32 = 0x1f7;
/// Banjo & Kazooie's Final Smash cinematic phase 3 status.
pub const BUDDY_FINAL_SCENE03: i32 = 0x1f8;
/// Banjo & Kazooie's Final Smash cinematic phase 4 status.
pub const BUDDY_FINAL_SCENE04: i32 = 0x1f9;
/// Banjo & Kazooie's up special jump phase status.
pub const BUDDY_SPECIAL_HI_JUMP: i32 = 0x1f2;
/// Captain Falcon's Final Smash ending status.
pub const CAPTAIN_FINAL_END: i32 = 0x1eb;
/// Captain Falcon's Final Smash ride status.
pub const CAPTAIN_FINAL_RIDE: i32 = 0x1e7;
/// Captain Falcon's Final Smash cinematic phase 1 status.
pub const CAPTAIN_FINAL_SCENE01: i32 = 0x1e8;
/// Captain Falcon's Final Smash cinematic phase 2 status.
pub const CAPTAIN_FINAL_SCENE02: i32 = 0x1e9;
/// Captain Falcon's Final Smash cinematic phase 3 status.
pub const CAPTAIN_FINAL_SCENE03: i32 = 0x1ea;
/// Captain Falcon's up special command-grab cling status.
pub const CAPTAIN_SPECIAL_HI_CLING: i32 = 0x1e3;
/// Captain Falcon's up special command-grab throw status.
pub const CAPTAIN_SPECIAL_HI_THROW: i32 = 0x1e4;
/// Cloud's up special variant 2 status.
pub const CLOUD_SPECIAL_HI2: i32 = 0x1e3;
/// Cloud's up special variant 2 fall phase status.
pub const CLOUD_SPECIAL_HI2_FALL: i32 = 0x1e4;
/// Cloud's up special variant 2 landing phase status.
pub const CLOUD_SPECIAL_HI2_LANDING: i32 = 0x1e5;
/// King Dedede's Final Smash ending status.
pub const DEDEDE_FINAL_END: i32 = 0x208;
/// King Dedede's Final Smash ready phase status.
pub const DEDEDE_FINAL_READY: i32 = 0x204;
/// King Dedede's Final Smash cinematic phase 1 status.
pub const DEDEDE_FINAL_SCENE01: i32 = 0x205;
/// King Dedede's Final Smash cinematic phase 2 status.
pub const DEDEDE_FINAL_SCENE02: i32 = 0x206;
/// King Dedede's Final Smash cinematic phase 3 status.
pub const DEDEDE_FINAL_SCENE03: i32 = 0x207;
/// King Dedede's up special failed recovery status.
pub const DEDEDE_SPECIAL_HI_FAILURE: i32 = 0x1f9;
/// King Dedede's up special hit phase status.
pub const DEDEDE_SPECIAL_HI_HIT: i32 = 0x1f8;
/// King Dedede's up special jump phase status.
pub const DEDEDE_SPECIAL_HI_JUMP: i32 = 0x1f6;
/// King Dedede's up special landing phase status.
pub const DEDEDE_SPECIAL_HI_LANDING: i32 = 0x1f7;
/// King Dedede's up special turnaround phase status.
pub const DEDEDE_SPECIAL_HI_TURN: i32 = 0x1fa;
/// Kazuya's Final Smash ending status.
pub const DEMON_FINAL_END: i32 = 0x212;
/// Kazuya's Final Smash ready phase status.
pub const DEMON_FINAL_READY: i32 = 0x210;
/// Kazuya's Final Smash cinematic phase 1 status.
pub const DEMON_FINAL_SCENE01: i32 = 0x211;
/// Kazuya's up special fall phase status.
pub const DEMON_SPECIAL_HI_FALL: i32 = 0x20c;
/// Kazuya's up special rising phase status.
pub const DEMON_SPECIAL_HI_RISE: i32 = 0x20b;
/// Diddy Kong's up special charge phase status.
pub const DIDDY_SPECIAL_HI_CHARGE: i32 = 0x1f0;
/// Diddy Kong's up special rolling fall status.
pub const DIDDY_SPECIAL_HI_FALL_ROLL: i32 = 0x1f5;
/// Diddy Kong's up special hit phase ceiling collision status.
pub const DIDDY_SPECIAL_HI_HIT_CEIL: i32 = 0x1f4;
/// Diddy Kong's up special upper status.
pub const DIDDY_SPECIAL_HI_UPPER: i32 = 0x1f2;
/// Terry's up special command status.
pub const DOLLY_SPECIAL_HI_COMMAND: i32 = 0x1f5;
/// Terry's up special fall phase status.
pub const DOLLY_SPECIAL_HI_FALL: i32 = 0x1f3;
/// Terry's up special jump phase status.
pub const DOLLY_SPECIAL_HI_JUMP: i32 = 0x1f2;
/// Terry's up special landing phase status.
pub const DOLLY_SPECIAL_HI_LANDING: i32 = 0x1f4;
/// Donkey Kong's Final Smash attack phase status.
pub const DONKEY_FINAL_ATTACK: i32 = 0x200;
/// Donkey Kong's Final Smash ending status.
pub const DONKEY_FINAL_END: i32 = 0x201;
/// Duck Hunt's up special ending status.
pub const DUCKHUNT_SPECIAL_HI_END: i32 = 0x1e2;
/// Duck Hunt's up special flight phase status.
pub const DUCKHUNT_SPECIAL_HI_FLY: i32 = 0x1e1;
/// Sephiroth's Final Smash ending status.
pub const EDGE_FINAL_END: i32 = 0x1f2;
/// Sephiroth's Final Smash ready phase status.
pub const EDGE_FINAL_READY: i32 = 0x1f0;
/// Sephiroth's Final Smash cinematic phase 1 status.
pub const EDGE_FINAL_SCENE01: i32 = 0x1f1;
/// Sephiroth's up special fully charged rush status.
pub const EDGE_SPECIAL_HI_CHARGED_RUSH: i32 = 0x1eb;
/// Sephiroth's up special ending status.
pub const EDGE_SPECIAL_HI_END: i32 = 0x1ed;
/// Sephiroth's up special landing phase status.
pub const EDGE_SPECIAL_HI_LANDING: i32 = 0x1ec;
/// Sephiroth's up special rush phase status.
pub const EDGE_SPECIAL_HI_RUSH: i32 = 0x1ea;
/// Pyra's Final Smash ending status.
pub const EFLAME_FINAL_END: i32 = 0x1e6;
/// Pyra's Final Smash ready phase status.
pub const EFLAME_FINAL_READY: i32 = 0x1e3;
/// Pyra's Final Smash cinematic phase 1 status.
pub const EFLAME_FINAL_SCENE01: i32 = 0x1e4;
/// Pyra's Final Smash cinematic phase 2 status.
pub const EFLAME_FINAL_SCENE02: i32 = 0x1e5;
/// Pyra's up special ending status.
pub const EFLAME_SPECIAL_HI_END: i32 = 0x1ed;
/// Pyra's up special jump phase status.
pub const EFLAME_SPECIAL_HI_JUMP: i32 = 0x1eb;
/// Pyra's up special looping phase status.
pub const EFLAME_SPECIAL_HI_LOOP: i32 = 0x1ec;
/// Mythra's Final Smash ending status.
pub const ELIGHT_FINAL_END: i32 = 0x1e6;
/// Mythra's Final Smash ready phase status.
pub const ELIGHT_FINAL_READY: i32 = 0x1e3;
/// Mythra's Final Smash cinematic phase 1 status.
pub const ELIGHT_FINAL_SCENE01: i32 = 0x1e4;
/// Mythra's Final Smash cinematic phase 2 status.
pub const ELIGHT_FINAL_SCENE02: i32 = 0x1e5;
/// Mythra's Final Smash cinematic phase 3 status.
pub const ELIGHT_FINAL_SCENE03: i32 = 0x1ef;
/// Mythra's Final Smash cinematic phase 4 status.
pub const ELIGHT_FINAL_SCENE04: i32 = 0x1f0;
/// Mythra's up special attack1 status.
pub const ELIGHT_SPECIAL_HI_ATTACK1: i32 = 0x1ec;
/// Mythra's up special attack2 status.
pub const ELIGHT_SPECIAL_HI_ATTACK2: i32 = 0x1ed;
/// Mythra's up special finisher status.
pub const ELIGHT_SPECIAL_HI_FINISH: i32 = 0x1ee;
/// Mythra's up special jump phase status.
pub const ELIGHT_SPECIAL_HI_JUMP: i32 = 0x1eb;
/// Falco's Final Smash ending status.
pub const FALCO_FINAL_END: i32 = 0x1e7;
/// Falco's Final Smash ready phase status.
pub const FALCO_FINAL_READY: i32 = 0x1e5;
/// Falco's Final Smash cinematic phase 1 status.
pub const FALCO_FINAL_SCENE01: i32 = 0x1e6;
/// Falco's up special bounce phase status.
pub const FALCO_SPECIAL_HI_BOUND: i32 = 0x1e4;
/// Falco's up special rush phase status.
pub const FALCO_SPECIAL_HI_RUSH: i32 = 0x1e2;
/// Falco's up special rush ending status.
pub const FALCO_SPECIAL_HI_RUSH_END: i32 = 0x1e3;
/// Fox's Final Smash ending status.
pub const FOX_FINAL_END: i32 = 0x1e9;
/// Fox's Final Smash ready phase status.
pub const FOX_FINAL_READY: i32 = 0x1e7;
/// Fox's Final Smash cinematic phase 1 status.
pub const FOX_FINAL_SCENE01: i32 = 0x1e8;
/// Fox's up special bounce phase status.
pub const FOX_SPECIAL_HI_BOUND: i32 = 0x1e3;
/// Fox's up special rush phase status.
pub const FOX_SPECIAL_HI_RUSH: i32 = 0x1e1;
/// Fox's up special rush ending status.
pub const FOX_SPECIAL_HI_RUSH_END: i32 = 0x1e2;
/// Mr. Game & Watch's up special close status.
pub const GAMEWATCH_SPECIAL_HI_CLOSE: i32 = 0x1e2;
/// Mr. Game & Watch's up special fall phase status.
pub const GAMEWATCH_SPECIAL_HI_FALL: i32 = 0x1e1;
/// Ganondorf's Final Smash attack phase status.
pub const GANON_FINAL_ATTACK: i32 = 0x1ea;
/// Ganondorf's Final Smash ending status.
pub const GANON_FINAL_END: i32 = 0x1eb;
/// Ganondorf's up special command-grab cling status.
pub const GANON_SPECIAL_HI_CLING: i32 = 0x1e6;
/// Ganondorf's up special command-grab throw status.
pub const GANON_SPECIAL_HI_THROW: i32 = 0x1e7;
/// Incineroar's up special bounce phase status.
pub const GAOGAEN_SPECIAL_HI_BOUND: i32 = 0x1ea;
/// Incineroar's up special ending status.
pub const GAOGAEN_SPECIAL_HI_END: i32 = 0x1e9;
/// Incineroar's up special fall phase status.
pub const GAOGAEN_SPECIAL_HI_FALL: i32 = 0x1e7;
/// Incineroar's up special looping phase status.
pub const GAOGAEN_SPECIAL_HI_LOOP: i32 = 0x1e8;
/// Incineroar's up special turnaround phase status.
pub const GAOGAEN_SPECIAL_HI_TURN: i32 = 0x1e6;
/// Greninja's Final Smash ending status.
pub const GEKKOUGA_FINAL_END: i32 = 0x1f0;
/// Greninja's Final Smash wait phase status.
pub const GEKKOUGA_FINAL_WAIT: i32 = 0x1ef;
/// Greninja's up special ending status.
pub const GEKKOUGA_SPECIAL_HI_END: i32 = 0x1e9;
/// Greninja's up special looping phase status.
pub const GEKKOUGA_SPECIAL_HI_LOOP: i32 = 0x1e7;
/// Ike's Final Smash attack phase status.
pub const IKE_FINAL_ATTACK: i32 = 0x1ef;
/// Ike's Final Smash ending status.
pub const IKE_FINAL_END: i32 = 0x1f1;
/// Ike's Final Smash fall phase status.
pub const IKE_FINAL_FALL: i32 = 0x1f0;
/// Ike's Final Smash hit phase status.
pub const IKE_FINAL_HIT: i32 = 0x1ed;
/// Ike's Final Smash movement phase status.
pub const IKE_FINAL_MOVE: i32 = 0x1ee;
/// Ike's up special phase 2 status.
pub const IKE_SPECIAL_HI_2: i32 = 0x1e9;
/// Ike's up special phase 3 status.
pub const IKE_SPECIAL_HI_3: i32 = 0x1ea;
/// Ike's up special phase 4 status.
pub const IKE_SPECIAL_HI_4: i32 = 0x1eb;
/// Inkling's up special fall phase status.
pub const INKLING_SPECIAL_HI_FALL: i32 = 0x1ee;
/// Inkling's up special jump phase status.
pub const INKLING_SPECIAL_HI_JUMP: i32 = 0x1eb;
/// Inkling's up special rotation phase status.
pub const INKLING_SPECIAL_HI_ROT: i32 = 0x1ec;
/// Inkling's up special ceiling collision status.
pub const INKLING_SPECIAL_HI_STOP_CEIL: i32 = 0x1ed;
/// Joker's Final Smash dash ending status.
pub const JACK_FINAL_DASH_END: i32 = 0x1f7;
/// Joker's Final Smash ending status.
pub const JACK_FINAL_END: i32 = 0x1fb;
/// Joker's Final Smash ready phase status.
pub const JACK_FINAL_READY: i32 = 0x1f8;
/// Joker's Final Smash cinematic phase 1 status.
pub const JACK_FINAL_SCENE01: i32 = 0x1f9;
/// Joker's Final Smash cinematic phase 2 status.
pub const JACK_FINAL_SCENE02: i32 = 0x1fa;
/// Joker's Final Smash turn-dash phase status.
pub const JACK_FINAL_TURN_DASH: i32 = 0x1f6;
/// Joker's up special variant 2 ending status.
pub const JACK_SPECIAL_HI2_END: i32 = 0x1f4;
/// Joker's up special variant 2 rush phase status.
pub const JACK_SPECIAL_HI2_RUSH: i32 = 0x1f3;
/// Joker's up special cut status.
pub const JACK_SPECIAL_HI_CUT: i32 = 0x1f2;
/// Joker's up special pickup status.
pub const JACK_SPECIAL_HI_PICKUP: i32 = 0x1f0;
/// Joker's up special pull status.
pub const JACK_SPECIAL_HI_PULL: i32 = 0x1ef;
/// Joker's up special command-grab throw status.
pub const JACK_SPECIAL_HI_THROW: i32 = 0x1f1;
/// Corrin's Final Smash ending status.
pub const KAMUI_FINAL_END: i32 = 0x1f3;
/// Corrin's Final Smash ready phase status.
pub const KAMUI_FINAL_READY: i32 = 0x1ef;
/// Corrin's Final Smash scene attack phase status.
pub const KAMUI_FINAL_SCENE_ATTACK: i32 = 0x1f1;
/// Corrin's Final Smash scene attack phase wait phase status.
pub const KAMUI_FINAL_SCENE_ATTACK_WAIT: i32 = 0x1f2;
/// Corrin's Final Smash scene entry phase status.
pub const KAMUI_FINAL_SCENE_ENTRY: i32 = 0x1f0;
/// Kirby's Final Smash attack phase status.
pub const KIRBY_FINAL_ATTACK: i32 = 0x203;
/// Kirby's Final Smash calling phase status.
pub const KIRBY_FINAL_CALLING: i32 = 0x201;
/// Kirby's Final Smash descending pot status.
pub const KIRBY_FINAL_DOWN_NABE: i32 = 0x204;
/// Kirby's Final Smash ending status.
pub const KIRBY_FINAL_END: i32 = 0x205;
/// Kirby's Final Smash rising pot status.
pub const KIRBY_FINAL_UP_NABE: i32 = 0x202;
/// Kirby's up special variant 2 status.
pub const KIRBY_SPECIAL_HI2: i32 = 0x1fc;
/// Kirby's up special variant 3 status.
pub const KIRBY_SPECIAL_HI3: i32 = 0x1fd;
/// Kirby's up special variant 4 status.
pub const KIRBY_SPECIAL_HI4: i32 = 0x1fe;
/// Bowser Jr.'s Final Smash draw status.
pub const KOOPAJR_FINAL_DRAW: i32 = 0x1ed;
/// Bowser Jr.'s Final Smash ending status.
pub const KOOPAJR_FINAL_END: i32 = 0x1ee;
/// Bowser Jr.'s up special attack phase status.
pub const KOOPAJR_SPECIAL_HI_ATTACK: i32 = 0x1e9;
/// Bowser Jr.'s up special escape status.
pub const KOOPAJR_SPECIAL_HI_ESCAPE: i32 = 0x1ea;
/// Bowser Jr.'s up special fall phase status.
pub const KOOPAJR_SPECIAL_HI_FALL: i32 = 0x1e8;
/// Bowser Jr.'s up special landing phase status.
pub const KOOPAJR_SPECIAL_HI_LANDING: i32 = 0x1ec;
/// Bowser Jr.'s up special release phase status.
pub const KOOPAJR_SPECIAL_HI_SHOOT: i32 = 0x1e7;
/// Bowser's Final Smash ending status.
pub const KOOPA_FINAL_END: i32 = 0x1ea;
/// Bowser's up special airborne status.
pub const KOOPA_SPECIAL_HI_A: i32 = 0x1e2;
/// Bowser's up special grounded status.
pub const KOOPA_SPECIAL_HI_G: i32 = 0x1e1;
/// King K. Rool's up special status.
pub const KROOL_SPECIAL_HI: i32 = 0x1eb;
/// King K. Rool's up special aerial ending status.
pub const KROOL_SPECIAL_HI_AIR_END: i32 = 0x1ec;
/// King K. Rool's up special fall phase status.
pub const KROOL_SPECIAL_HI_FALL: i32 = 0x1ed;
/// King K. Rool's up special landing phase status.
pub const KROOL_SPECIAL_HI_LANDING: i32 = 0x1ee;
/// King K. Rool's up special startup status.
pub const KROOL_SPECIAL_HI_START: i32 = 0x1ea;
/// Link's Final Smash combo status.
pub const LINK_FINAL_COMBO: i32 = 0x1e7;
/// Link's Final Smash dash phase status.
pub const LINK_FINAL_DASH: i32 = 0x1e6;
/// Link's up special ending status.
pub const LINK_SPECIAL_HI_END: i32 = 0x1e3;
/// Link's up special charge hold status.
pub const LINK_SPECIAL_HI_HOLD: i32 = 0x1e2;
/// Little Mac's Final Smash attack phase status.
pub const LITTLEMAC_FINAL_ATTACK: i32 = 0x1f2;
/// Little Mac's Final Smash dash phase status.
pub const LITTLEMAC_FINAL_DASH: i32 = 0x1ef;
/// Little Mac's Final Smash dash ending status.
pub const LITTLEMAC_FINAL_DASH_END: i32 = 0x1f0;
/// Little Mac's Final Smash ending status.
pub const LITTLEMAC_FINAL_END: i32 = 0x1f3;
/// Little Mac's Final Smash hit phase status.
pub const LITTLEMAC_FINAL_HIT: i32 = 0x1f1;
/// Little Mac's up special jump phase status.
pub const LITTLEMAC_SPECIAL_HI_JUMP: i32 = 0x1ed;
/// Little Mac's up special startup status.
pub const LITTLEMAC_SPECIAL_HI_START: i32 = 0x1ec;
/// Lucario's Final Smash attack phase status.
pub const LUCARIO_FINAL_ATTACK: i32 = 0x1ee;
/// Lucario's Final Smash ending status.
pub const LUCARIO_FINAL_END: i32 = 0x1ef;
/// Lucario's Final Smash entry phase status.
pub const LUCARIO_FINAL_ENTRY: i32 = 0x1ec;
/// Lucario's Final Smash charge hold status.
pub const LUCARIO_FINAL_HOLD: i32 = 0x1ed;
/// Lucario's up special bounce phase status.
pub const LUCARIO_SPECIAL_HI_BOUND: i32 = 0x1e8;
/// Lucario's up special rush phase status.
pub const LUCARIO_SPECIAL_HI_RUSH: i32 = 0x1e6;
/// Lucario's up special rush ending status.
pub const LUCARIO_SPECIAL_HI_RUSH_END: i32 = 0x1e7;
/// Lucas's Final Smash attack phase status.
pub const LUCAS_FINAL_ATTACK: i32 = 0x1ec;
/// Lucas's up special again status.
pub const LUCAS_SPECIAL_HI_AGAIN: i32 = 0x1e8;
/// Lucas's up special attack phase status.
pub const LUCAS_SPECIAL_HI_ATTACK: i32 = 0x1e6;
/// Lucas's up special ending status.
pub const LUCAS_SPECIAL_HI_END: i32 = 0x1e5;
/// Lucas's up special charge hold status.
pub const LUCAS_SPECIAL_HI_HOLD: i32 = 0x1e4;
/// Lucas's up special reflect phase status.
pub const LUCAS_SPECIAL_HI_REFLECT: i32 = 0x1e7;
/// Luigi's Final Smash ending status.
pub const LUIGI_FINAL_END: i32 = 0x1ec;
/// Luigi's Final Smash failed phase status.
pub const LUIGI_FINAL_FAIL: i32 = 0x1ed;
/// Luigi's Final Smash release phase status.
pub const LUIGI_FINAL_SHOOT: i32 = 0x1eb;
/// Luigi's Final Smash shot preparation status.
pub const LUIGI_FINAL_SHOOT_READY: i32 = 0x1ea;
/// Luigi's Final Smash vacuum phase status.
pub const LUIGI_FINAL_VACUUM: i32 = 0x1e9;
/// Luigi's up special drop status.
pub const LUIGI_SPECIAL_HI_DROP: i32 = 0x1e6;
/// Luigi's up special fall phase status.
pub const LUIGI_SPECIAL_HI_FALL: i32 = 0x1e7;
/// Luigi's up special landing phase fall phase status.
pub const LUIGI_SPECIAL_HI_LANDING_FALL: i32 = 0x1e8;
/// Marth's Final Smash dash phase status.
pub const MARTH_FINAL_DASH: i32 = 0x1e8;
/// Marth's Final Smash dash ending status.
pub const MARTH_FINAL_DASH_END: i32 = 0x1e9;
/// Marth's Final Smash ending status.
pub const MARTH_FINAL_END: i32 = 0x1ea;
/// Byleth's Final Smash ending status.
pub const MASTER_FINAL_END: i32 = 0x1f7;
/// Byleth's Final Smash ready phase status.
pub const MASTER_FINAL_READY: i32 = 0x1f3;
/// Byleth's Final Smash cinematic phase 1 status.
pub const MASTER_FINAL_SCENE01: i32 = 0x1f4;
/// Byleth's Final Smash cinematic phase 2 status.
pub const MASTER_FINAL_SCENE02: i32 = 0x1f5;
/// Byleth's Final Smash cinematic phase 3 status.
pub const MASTER_FINAL_SCENE03: i32 = 0x1f6;
/// Byleth's up special failed phase status.
pub const MASTER_SPECIAL_HI_FAIL: i32 = 0x1ed;
/// Byleth's up special hit phase status.
pub const MASTER_SPECIAL_HI_HIT: i32 = 0x1ea;
/// Byleth's up special overtake status.
pub const MASTER_SPECIAL_HI_OVERTAKE: i32 = 0x1eb;
/// Byleth's up special wall jump phase status.
pub const MASTER_SPECIAL_HI_WALL_JUMP: i32 = 0x1ec;
/// Meta Knight's Final Smash attack phase status.
pub const METAKNIGHT_FINAL_ATTACK: i32 = 0x1ec;
/// Meta Knight's Final Smash ending status.
pub const METAKNIGHT_FINAL_END: i32 = 0x1ed;
/// Meta Knight's Final Smash hit-confirm wait status.
pub const METAKNIGHT_FINAL_HIT_WAIT: i32 = 0x1e9;
/// Meta Knight's Final Smash jump phase status.
pub const METAKNIGHT_FINAL_JUMP: i32 = 0x1eb;
/// Meta Knight's Final Smash squat status.
pub const METAKNIGHT_FINAL_SQUAT: i32 = 0x1ea;
/// Meta Knight's up special looping phase status.
pub const METAKNIGHT_SPECIAL_HI_LOOP: i32 = 0x1e8;
/// Mewtwo's Final Smash ending status.
pub const MEWTWO_FINAL_END: i32 = 0x1e8;
/// Mewtwo's up special phase 2 status.
pub const MEWTWO_SPECIAL_HI_2: i32 = 0x1e6;
/// Mewtwo's up special phase 3 status.
pub const MEWTWO_SPECIAL_HI_3: i32 = 0x1e7;
/// Mii Brawler enemy's up special variant 2 phase 2 status.
pub const MIIENEMYF_SPECIAL_HI2_2: i32 = 0x1ee;
/// Mii Brawler enemy's up special variant 2 phase 3 status.
pub const MIIENEMYF_SPECIAL_HI2_3: i32 = 0x1ef;
/// Mii Brawler enemy's up special variant 2 phase 4 status.
pub const MIIENEMYF_SPECIAL_HI2_4: i32 = 0x1f0;
/// Mii Brawler enemy's up special variant 3 attack phase status.
pub const MIIENEMYF_SPECIAL_HI3_ATTACK: i32 = 0x1f1;
/// Mii Brawler enemy's up special variant 3 ending status.
pub const MIIENEMYF_SPECIAL_HI3_END: i32 = 0x1f2;
/// Mii Gunner enemy's up special variant 1 jump phase status.
pub const MIIENEMYG_SPECIAL_HI1_JUMP: i32 = 0x1ed;
/// Mii Gunner enemy's up special variant 2 failed phase status.
pub const MIIENEMYG_SPECIAL_HI2_FAIL: i32 = 0x1ee;
/// Mii Gunner enemy's up special variant 3 rush phase status.
pub const MIIENEMYG_SPECIAL_HI3_RUSH: i32 = 0x1ef;
/// Mii Gunner enemy's up special variant 3 rush ending status.
pub const MIIENEMYG_SPECIAL_HI3_RUSH_END: i32 = 0x1f0;
/// Mii Swordfighter enemy's Final Smash ending status.
pub const MIIENEMYS_FINAL_END: i32 = 0x1f9;
/// Mii Swordfighter enemy's Final Smash charge hold status.
pub const MIIENEMYS_FINAL_HOLD: i32 = 0x1f8;
/// Mii Swordfighter enemy's up special variant 1 ending status.
pub const MIIENEMYS_SPECIAL_HI1_END: i32 = 0x1ef;
/// Mii Swordfighter enemy's up special variant 1 charge hold status.
pub const MIIENEMYS_SPECIAL_HI1_HOLD: i32 = 0x1ee;
/// Mii Swordfighter enemy's up special variant 2 bounce phase status.
pub const MIIENEMYS_SPECIAL_HI2_BOUND: i32 = 0x1f2;
/// Mii Swordfighter enemy's up special variant 2 rush phase status.
pub const MIIENEMYS_SPECIAL_HI2_RUSH: i32 = 0x1f0;
/// Mii Swordfighter enemy's up special variant 3 ending status.
pub const MIIENEMYS_SPECIAL_HI3_END: i32 = 0x1f5;
/// Mii Swordfighter enemy's up special variant 3 looping phase status.
pub const MIIENEMYS_SPECIAL_HI3_LOOP: i32 = 0x1f4;
/// Mii Brawler's Final Smash attack phase status.
pub const MIIFIGHTER_FINAL_ATTACK: i32 = 0x205;
/// Mii Brawler's Final Smash ending status.
pub const MIIFIGHTER_FINAL_END: i32 = 0x206;
/// Mii Brawler's Final Smash hit phase status.
pub const MIIFIGHTER_FINAL_HIT: i32 = 0x203;
/// Mii Brawler's Final Smash movement phase status.
pub const MIIFIGHTER_FINAL_MOVE: i32 = 0x204;
/// Mii Brawler's up special variant 1 phase 2 status.
pub const MIIFIGHTER_SPECIAL_HI1_2: i32 = 0x1f4;
/// Mii Brawler's up special variant 1 phase 3 status.
pub const MIIFIGHTER_SPECIAL_HI1_3: i32 = 0x1f5;
/// Mii Brawler's up special variant 1 phase 4 status.
pub const MIIFIGHTER_SPECIAL_HI1_4: i32 = 0x1f6;
/// Mii Brawler's up special variant 2 attack phase status.
pub const MIIFIGHTER_SPECIAL_HI2_ATTACK: i32 = 0x1f7;
/// Mii Brawler's up special variant 2 ending status.
pub const MIIFIGHTER_SPECIAL_HI2_END: i32 = 0x1f8;
/// Mii Gunner's Final Smash ending status.
pub const MIIGUNNER_FINAL_END: i32 = 0x1f9;
/// Mii Gunner's Final Smash looping phase status.
pub const MIIGUNNER_FINAL_LOOP: i32 = 0x1f8;
/// Mii Gunner's up special variant 2 jump phase status.
pub const MIIGUNNER_SPECIAL_HI2_JUMP: i32 = 0x1ef;
/// Mii Gunner's up special variant 3 rush phase status.
pub const MIIGUNNER_SPECIAL_HI3_RUSH: i32 = 0x1f0;
/// Mii Gunner's up special variant 3 rush ending status.
pub const MIIGUNNER_SPECIAL_HI3_RUSH_END: i32 = 0x1f1;
/// Mii Swordfighter's Final Smash ending status.
pub const MIISWORDSMAN_FINAL_END: i32 = 0x1f8;
/// Mii Swordfighter's Final Smash charge hold status.
pub const MIISWORDSMAN_FINAL_HOLD: i32 = 0x1f7;
/// Villager's Final Smash cheer status.
pub const MURABITO_FINAL_CHEER: i32 = 0x201;
/// Villager's Final Smash ending status.
pub const MURABITO_FINAL_END: i32 = 0x203;
/// Villager's Final Smash happy status.
pub const MURABITO_FINAL_HAPPY: i32 = 0x202;
/// Villager's Final Smash money status.
pub const MURABITO_FINAL_MONEY: i32 = 0x200;
/// Villager's Final Smash surprise status.
pub const MURABITO_FINAL_SURPRISE: i32 = 0x1ff;
/// Ness's Final Smash attack phase status.
pub const NESS_FINAL_ATTACK: i32 = 0x1ec;
/// Ness's up special again status.
pub const NESS_SPECIAL_HI_AGAIN: i32 = 0x1e8;
/// Ness's up special attack phase status.
pub const NESS_SPECIAL_HI_ATTACK: i32 = 0x1e6;
/// Ness's up special ending status.
pub const NESS_SPECIAL_HI_END: i32 = 0x1e5;
/// Ness's up special charge hold status.
pub const NESS_SPECIAL_HI_HOLD: i32 = 0x1e4;
/// Ness's up special reflect phase status.
pub const NESS_SPECIAL_HI_REFLECT: i32 = 0x1e7;
/// Piranha Plant's up special ending status.
pub const PACKUN_SPECIAL_HI_END: i32 = 0x1ec;
/// Piranha Plant's up special landing phase status.
pub const PACKUN_SPECIAL_HI_LANDING: i32 = 0x1ed;
/// Pac-Man's Final Smash ending status.
pub const PACMAN_FINAL_END: i32 = 0x1ef;
/// Pac-Man's Final Smash movement phase status.
pub const PACMAN_FINAL_MOVE: i32 = 0x1ee;
/// Pac-Man's up special ending status.
pub const PACMAN_SPECIAL_HI_END: i32 = 0x1ed;
/// Pac-Man's up special looping phase status.
pub const PACMAN_SPECIAL_HI_LOOP: i32 = 0x1ec;
/// Palutena's Final Smash beam status.
pub const PALUTENA_FINAL_BEAM: i32 = 0x1e7;
/// Palutena's Final Smash beam ending status.
pub const PALUTENA_FINAL_BEAM_END: i32 = 0x1e8;
/// Palutena's Final Smash beam startup status.
pub const PALUTENA_FINAL_BEAM_START: i32 = 0x1e6;
/// Palutena's Final Smash wait phase status.
pub const PALUTENA_FINAL_WAIT: i32 = 0x1e5;
/// Palutena's up special phase 2 status.
pub const PALUTENA_SPECIAL_HI_2: i32 = 0x1e1;
/// Palutena's up special phase 3 status.
pub const PALUTENA_SPECIAL_HI_3: i32 = 0x1e2;
/// Peach's up special aerial ending status.
pub const PEACH_SPECIAL_HI_AIR_END: i32 = 0x1e5;
/// Peach's up special fall phase status.
pub const PEACH_SPECIAL_HI_FALL: i32 = 0x1e4;
/// Peach's up special landing phase status.
pub const PEACH_SPECIAL_HI_LANDING: i32 = 0x1e6;
/// Steve's Final Smash ending status.
pub const PICKEL_FINAL_END: i32 = 0x21a;
/// Steve's Final Smash ready phase status.
pub const PICKEL_FINAL_READY: i32 = 0x218;
/// Steve's Final Smash cinematic phase 1 status.
pub const PICKEL_FINAL_SCENE01: i32 = 0x219;
/// Steve's up special fall phase status.
pub const PICKEL_SPECIAL_HI_FALL: i32 = 0x20e;
/// Steve's up special fall phase special status.
pub const PICKEL_SPECIAL_HI_FALL_SPECIAL: i32 = 0x20f;
/// Steve's up special gliding status.
pub const PICKEL_SPECIAL_HI_GLIDING: i32 = 0x20c;
/// Steve's up special jump phase status.
pub const PICKEL_SPECIAL_HI_JUMP: i32 = 0x20a;
/// Steve's up special landing phase status.
pub const PICKEL_SPECIAL_HI_LANDING: i32 = 0x20d;
/// Steve's up special wait phase status.
pub const PICKEL_SPECIAL_HI_WAIT: i32 = 0x20b;
/// Pikachu's up special ending status.
pub const PIKACHU_SPECIAL_HI_END: i32 = 0x1e2;
/// Pikachu's up special warp status.
pub const PIKACHU_SPECIAL_HI_WARP: i32 = 0x1e1;
/// Olimar's Final Smash ending status.
pub const PIKMIN_FINAL_END: i32 = 0x1e4;
/// Olimar's Final Smash fall phase status.
pub const PIKMIN_FINAL_FALL: i32 = 0x1e3;
/// Olimar's Final Smash flight phase status.
pub const PIKMIN_FINAL_FLY: i32 = 0x1e1;
/// Olimar's Final Smash wait phase status.
pub const PIKMIN_FINAL_WAIT: i32 = 0x1e2;
/// Olimar's up special ending status.
pub const PIKMIN_SPECIAL_HI_END: i32 = 0x1e7;
/// Olimar's up special landing phase status.
pub const PIKMIN_SPECIAL_HI_LANDING: i32 = 0x1e6;
/// Olimar's up special wait phase status.
pub const PIKMIN_SPECIAL_HI_WAIT: i32 = 0x1e5;
/// Pit's Final Smash attack phase status.
pub const PIT_FINAL_ATTACK: i32 = 0x1ec;
/// Pit's Final Smash ending status.
pub const PIT_FINAL_END: i32 = 0x1ed;
/// Pit's Final Smash wait phase status.
pub const PIT_FINAL_WAIT: i32 = 0x1eb;
/// Pit's up special rush phase status.
pub const PIT_SPECIAL_HI_RUSH: i32 = 0x1e7;
/// Pit's up special rush ending status.
pub const PIT_SPECIAL_HI_RUSH_END: i32 = 0x1e8;
/// Ice Climbers's Final Smash hang status.
pub const POPO_FINAL_HANG: i32 = 0x1ee;
/// Ice Climbers's Final Smash partner phase status.
pub const POPO_FINAL_PARTNER: i32 = 0x1ed;
/// Jigglypuff's Final Smash ending status.
pub const PURIN_FINAL_END: i32 = 0x1e9;
/// Jigglypuff's Final Smash wait phase status.
pub const PURIN_FINAL_WAIT: i32 = 0x1e8;
/// Robin's up special phase 2 status.
pub const REFLET_SPECIAL_HI_2: i32 = 0x1e8;
/// Robin's up special failed phase status.
pub const REFLET_SPECIAL_HI_FAIL: i32 = 0x1e9;
/// R.O.B.'s Final Smash ending status.
pub const ROBOT_FINAL_END: i32 = 0x1ea;
/// R.O.B.'s Final Smash looping phase status.
pub const ROBOT_FINAL_LOOP: i32 = 0x1e9;
/// R.O.B.'s up special attack phase status.
pub const ROBOT_SPECIAL_HI_ATTACK: i32 = 0x1e4;
/// R.O.B.'s up special item release phase aerial status.
pub const ROBOT_SPECIAL_HI_ITEM_SHOOT_AIR: i32 = 0x1e6;
/// R.O.B.'s up special item command-grab throw status.
pub const ROBOT_SPECIAL_HI_ITEM_THROW: i32 = 0x1e5;
/// R.O.B.'s up special keep status.
pub const ROBOT_SPECIAL_HI_KEEP: i32 = 0x1e3;
/// Mega Man's Final Smash ending status.
pub const ROCKMAN_FINAL_END: i32 = 0x1ef;
/// Mega Man's Final Smash inhale status.
pub const ROCKMAN_FINAL_INHALE: i32 = 0x1eb;
/// Mega Man's Final Smash cinematic phase 1 status.
pub const ROCKMAN_FINAL_SCENE01: i32 = 0x1ec;
/// Mega Man's Final Smash cinematic phase 2 status.
pub const ROCKMAN_FINAL_SCENE02: i32 = 0x1ed;
/// Mega Man's Final Smash cinematic phase 5 status.
pub const ROCKMAN_FINAL_SCENE05: i32 = 0x1ee;
/// Mega Man's up special jump phase status.
pub const ROCKMAN_SPECIAL_HI_JUMP: i32 = 0x1e2;
/// Rosalina & Luma's up special ending status.
pub const ROSETTA_SPECIAL_HI_END: i32 = 0x1e5;
/// Rosalina & Luma's up special jump phase status.
pub const ROSETTA_SPECIAL_HI_JUMP: i32 = 0x1e4;
/// Roy's Final Smash attack phase status.
pub const ROY_FINAL_ATTACK: i32 = 0x1f1;
/// Roy's Final Smash dash phase status.
pub const ROY_FINAL_DASH: i32 = 0x1ef;
/// Roy's Final Smash dash ending status.
pub const ROY_FINAL_DASH_END: i32 = 0x1f0;
/// Roy's Final Smash ending status.
pub const ROY_FINAL_END: i32 = 0x1ee;
/// Roy's up special phase 2 status.
pub const ROY_SPECIAL_HI_2: i32 = 0x1ea;
/// Roy's up special phase 3 status.
pub const ROY_SPECIAL_HI_3: i32 = 0x1eb;
/// Roy's up special phase 4 status.
pub const ROY_SPECIAL_HI_4: i32 = 0x1ec;
/// Ryu's up special command status.
pub const RYU_SPECIAL_HI_COMMAND: i32 = 0x1f4;
/// Ryu's up special fall phase status.
pub const RYU_SPECIAL_HI_FALL: i32 = 0x1f1;
/// Ryu's up special jump phase status.
pub const RYU_SPECIAL_HI_JUMP: i32 = 0x1f0;
/// Ryu's up special landing phase status.
pub const RYU_SPECIAL_HI_LANDING: i32 = 0x1f2;
/// Sheik's up special ending status.
pub const SHEIK_SPECIAL_HI_END: i32 = 0x1e8;
/// Sheik's up special movement phase status.
pub const SHEIK_SPECIAL_HI_MOVE: i32 = 0x1e7;
/// Shulk's Final Smash ending status.
pub const SHULK_FINAL_END: i32 = 0x1ec;
/// Shulk's Final Smash ready phase status.
pub const SHULK_FINAL_READY: i32 = 0x1e9;
/// Shulk's Final Smash scene attack phase status.
pub const SHULK_FINAL_SCENE_ATTACK: i32 = 0x1eb;
/// Shulk's Final Smash scene entry phase status.
pub const SHULK_FINAL_SCENE_ENTRY: i32 = 0x1ea;
/// Shulk's up special add status.
pub const SHULK_SPECIAL_HI_ADD: i32 = 0x1e5;
/// Simon's Final Smash ending status.
pub const SIMON_FINAL_END: i32 = 0x1eb;
/// Simon's Final Smash ready phase status.
pub const SIMON_FINAL_READY: i32 = 0x1e7;
/// Simon's Final Smash cinematic phase 1 status.
pub const SIMON_FINAL_SCENE01: i32 = 0x1e8;
/// Simon's Final Smash cinematic phase 2 status.
pub const SIMON_FINAL_SCENE02: i32 = 0x1e9;
/// Simon's Final Smash cinematic phase 3 status.
pub const SIMON_FINAL_SCENE03: i32 = 0x1ea;
/// Snake's Final Smash appeal status.
pub const SNAKE_FINAL_APPEAL: i32 = 0x200;
/// Snake's Final Smash camera phase status.
pub const SNAKE_FINAL_CAMERA_ZOOM: i32 = 0x1fe;
/// Snake's Final Smash ending status.
pub const SNAKE_FINAL_END: i32 = 0x1ff;
/// Snake's Final Smash lock on status.
pub const SNAKE_FINAL_LOCK_ON: i32 = 0x1fd;
/// Snake's up special cut status.
pub const SNAKE_SPECIAL_HI_CUT: i32 = 0x1f5;
/// Snake's up special hang status.
pub const SNAKE_SPECIAL_HI_HANG: i32 = 0x1f4;
/// Sonic's Final Smash ending status.
pub const SONIC_FINAL_END: i32 = 0x1f2;
/// Sonic's up special jump phase status.
pub const SONIC_SPECIAL_HI_JUMP: i32 = 0x1ef;
/// Zero Suit Samus's Final Smash ending status.
pub const SZEROSUIT_FINAL_END: i32 = 0x1eb;
/// Zero Suit Samus's Final Smash lock on status.
pub const SZEROSUIT_FINAL_LOCK_ON: i32 = 0x1ea;
/// Min Min's Final Smash ending status.
pub const TANTAN_FINAL_END: i32 = 0x1fe;
/// Min Min's Final Smash ready phase status.
pub const TANTAN_FINAL_READY: i32 = 0x1fa;
/// Min Min's Final Smash cinematic phase 1 status.
pub const TANTAN_FINAL_SCENE01: i32 = 0x1fb;
/// Min Min's Final Smash cinematic phase 2 status.
pub const TANTAN_FINAL_SCENE02: i32 = 0x1fc;
/// Min Min's Final Smash cinematic phase 3 status.
pub const TANTAN_FINAL_SCENE03: i32 = 0x1fd;
/// Min Min's up special aerial status.
pub const TANTAN_SPECIAL_HI_AIR: i32 = 0x1f7;
/// Min Min's up special aerial ending status.
pub const TANTAN_SPECIAL_HI_AIR_END: i32 = 0x1f9;
/// Min Min's up special aerial extended reach status.
pub const TANTAN_SPECIAL_HI_AIR_REACH: i32 = 0x1f8;
/// Min Min's up special ground status.
pub const TANTAN_SPECIAL_HI_GROUND: i32 = 0x1f4;
/// Min Min's up special ground ending status.
pub const TANTAN_SPECIAL_HI_GROUND_END: i32 = 0x1f6;
/// Min Min's up special ground jump phase status.
pub const TANTAN_SPECIAL_HI_GROUND_JUMP: i32 = 0x1f5;
/// Sora's Final Smash ending status.
pub const TRAIL_FINAL_END: i32 = 0x1f4;
/// Sora's Final Smash ready phase status.
pub const TRAIL_FINAL_READY: i32 = 0x1ef;
/// Sora's Final Smash cinematic phase 1 status.
pub const TRAIL_FINAL_SCENE01: i32 = 0x1f0;
/// Sora's Final Smash cinematic phase 2 status.
pub const TRAIL_FINAL_SCENE02: i32 = 0x1f1;
/// Sora's Final Smash cinematic phase 3 status.
pub const TRAIL_FINAL_SCENE03: i32 = 0x1f2;
/// Sora's Final Smash cinematic phase 4 status.
pub const TRAIL_FINAL_SCENE04: i32 = 0x1f3;
/// Wario's Final Smash dash phase status.
pub const WARIO_FINAL_DASH: i32 = 0x1fb;
/// Wario's Final Smash dash ending status.
pub const WARIO_FINAL_DASH_END: i32 = 0x1fc;
/// Wario's Final Smash ending status.
pub const WARIO_FINAL_END: i32 = 0x1ff;
/// Wario's Final Smash ready phase status.
pub const WARIO_FINAL_READY: i32 = 0x1fd;
/// Wario's Final Smash scene status.
pub const WARIO_FINAL_SCENE: i32 = 0x1fe;
/// Wario's up special jump phase status.
pub const WARIO_SPECIAL_HI_JUMP: i32 = 0x1f9;
/// Wii Fit Trainer's Final Smash ending status.
pub const WIIFIT_FINAL_END: i32 = 0x1ee;
/// Wii Fit Trainer's Final Smash charge hold status.
pub const WIIFIT_FINAL_HOLD: i32 = 0x1ed;
/// Wii Fit Trainer's up special ending status.
pub const WIIFIT_SPECIAL_HI_END: i32 = 0x1e9;
/// Wii Fit Trainer's up special jump phase status.
pub const WIIFIT_SPECIAL_HI_JUMP: i32 = 0x1e8;
/// Wolf's Final Smash control status.
pub const WOLF_FINAL_CONTROL: i32 = 0x1e8;
/// Wolf's Final Smash ending status.
pub const WOLF_FINAL_END: i32 = 0x1e9;
/// Wolf's Final Smash jump phase status.
pub const WOLF_FINAL_JUMP: i32 = 0x1e7;
/// Wolf's up special bounce phase status.
pub const WOLF_SPECIAL_HI_BOUND: i32 = 0x1e3;
/// Wolf's up special rush phase status.
pub const WOLF_SPECIAL_HI_RUSH: i32 = 0x1e1;
/// Wolf's up special rush ending status.
pub const WOLF_SPECIAL_HI_RUSH_END: i32 = 0x1e2;
/// Yoshi's Final Smash dash phase status.
pub const YOSHI_FINAL_DASH: i32 = 0x1ea;
/// Yoshi's Final Smash dash ending status.
pub const YOSHI_FINAL_DASH_END: i32 = 0x1eb;
/// Yoshi's Final Smash ending status.
pub const YOSHI_FINAL_END: i32 = 0x1ee;
/// Yoshi's Final Smash hit phase status.
pub const YOSHI_FINAL_HIT: i32 = 0x1ec;
/// Yoshi's Final Smash cinematic phase 1 status.
pub const YOSHI_FINAL_SCENE01: i32 = 0x1ed;
/// Zelda's Final Smash ending status.
pub const ZELDA_FINAL_END: i32 = 0x1ea;
/// Zelda's Final Smash finisher status.
pub const ZELDA_FINAL_FINISH: i32 = 0x1e9;
/// Zelda's Final Smash looping phase status.
pub const ZELDA_FINAL_LOOP: i32 = 0x1e8;
/// Zelda's Final Smash startup status.
pub const ZELDA_FINAL_START: i32 = 0x1e7;
/// Zelda's up special phase 2 status.
pub const ZELDA_SPECIAL_HI_2: i32 = 0x1e3;
/// Zelda's up special phase 3 status.
pub const ZELDA_SPECIAL_HI_3: i32 = 0x1e4;
/// Cloud's second Final Smash status.
pub const CLOUD_FINAL2: i32 = 0x1ef;
/// Cloud's second Final Smash attack phase status.
pub const CLOUD_FINAL2_ATTACK: i32 = 0x1f4;
/// Cloud's second Final Smash dash phase status.
pub const CLOUD_FINAL2_DASH: i32 = 0x1f0;
/// Cloud's second Final Smash dash ending status.
pub const CLOUD_FINAL2_DASH_END: i32 = 0x1f1;
/// Cloud's second Final Smash ending status.
pub const CLOUD_FINAL2_END: i32 = 0x1f6;
/// Cloud's second Final Smash fall phase status.
pub const CLOUD_FINAL2_FALL: i32 = 0x1f5;
/// Cloud's second Final Smash hit phase status.
pub const CLOUD_FINAL2_HIT: i32 = 0x1f2;
/// Cloud's second Final Smash movement phase status.
pub const CLOUD_FINAL2_MOVE: i32 = 0x1f3;
/// Cloud's Final Smash attack phase status.
pub const CLOUD_FINAL_ATTACK: i32 = 0x1ec;
/// Cloud's Final Smash dash phase status.
pub const CLOUD_FINAL_DASH: i32 = 0x1e8;
/// Cloud's Final Smash dash ending status.
pub const CLOUD_FINAL_DASH_END: i32 = 0x1e9;
/// Cloud's Final Smash ending status.
pub const CLOUD_FINAL_END: i32 = 0x1ee;
/// Cloud's Final Smash fall phase status.
pub const CLOUD_FINAL_FALL: i32 = 0x1ed;
/// Cloud's Final Smash hit phase status.
pub const CLOUD_FINAL_HIT: i32 = 0x1ea;
/// Cloud's Final Smash movement phase status.
pub const CLOUD_FINAL_MOVE: i32 = 0x1eb;
/// Diddy Kong's Final Smash charge phase status.
pub const DIDDY_FINAL_CHARGE: i32 = 0x1fb;
/// Diddy Kong's Final Smash ending status.
pub const DIDDY_FINAL_END: i32 = 0x1fe;
/// Diddy Kong's Final Smash finisher attack phase status.
pub const DIDDY_FINAL_FINISH_ATTACK: i32 = 0x1fc;
/// Diddy Kong's Final Smash flight phase status.
pub const DIDDY_FINAL_FLY: i32 = 0x1f9;
/// Diddy Kong's Final Smash ready phase charge phase status.
pub const DIDDY_FINAL_READY_CHARGE: i32 = 0x1fa;
/// Diddy Kong's Final Smash ready phase ending status.
pub const DIDDY_FINAL_READY_END: i32 = 0x1fd;
/// Diddy Kong's Final Smash startup status.
pub const DIDDY_FINAL_START: i32 = 0x1f7;
/// Diddy Kong's Final Smash wait phase flight phase status.
pub const DIDDY_FINAL_WAIT_FLY: i32 = 0x1f8;
/// Terry's Final Smash ending status.
pub const DOLLY_FINAL_END: i32 = 0x202;
/// Terry's Final Smash ready phase status.
pub const DOLLY_FINAL_READY: i32 = 0x1fc;
/// Terry's Final Smash cinematic phase 1 status.
pub const DOLLY_FINAL_SCENE01: i32 = 0x1fd;
/// Terry's Final Smash cinematic phase 2 status.
pub const DOLLY_FINAL_SCENE02: i32 = 0x1fe;
/// Terry's Final Smash cinematic phase 3 status.
pub const DOLLY_FINAL_SCENE03: i32 = 0x1ff;
/// Terry's Final Smash cinematic phase 4 status.
pub const DOLLY_FINAL_SCENE04: i32 = 0x200;
/// Terry's Final Smash cinematic phase 5 status.
pub const DOLLY_FINAL_SCENE05: i32 = 0x201;
/// Duck Hunt's Final Smash ending status.
pub const DUCKHUNT_FINAL_END: i32 = 0x1ea;
/// Duck Hunt's Final Smash ready phase status.
pub const DUCKHUNT_FINAL_READY: i32 = 0x1e4;
/// Duck Hunt's Final Smash cinematic phase 1 status.
pub const DUCKHUNT_FINAL_SCENE01: i32 = 0x1e5;
/// Duck Hunt's Final Smash cinematic phase 2 status.
pub const DUCKHUNT_FINAL_SCENE02: i32 = 0x1e6;
/// Duck Hunt's Final Smash cinematic phase 3 status.
pub const DUCKHUNT_FINAL_SCENE03: i32 = 0x1e7;
/// Duck Hunt's Final Smash cinematic phase 4 status.
pub const DUCKHUNT_FINAL_SCENE04: i32 = 0x1e8;
/// Duck Hunt's Final Smash cinematic phase 5 status.
pub const DUCKHUNT_FINAL_SCENE05: i32 = 0x1e9;
/// Mr. Game & Watch's Final Smash dash phase status.
pub const GAMEWATCH_FINAL_DASH: i32 = 0x1eb;
/// Mr. Game & Watch's Final Smash ending status.
pub const GAMEWATCH_FINAL_END: i32 = 0x204;
/// Mr. Game & Watch's Final Smash fall phase status.
pub const GAMEWATCH_FINAL_FALL: i32 = 0x1f4;
/// Mr. Game & Watch's Final Smash aerial fall status.
pub const GAMEWATCH_FINAL_FALL_AERIAL: i32 = 0x1f5;
/// Mr. Game & Watch's Final Smash jump phase status.
pub const GAMEWATCH_FINAL_JUMP: i32 = 0x1f2;
/// Mr. Game & Watch's Final Smash midair jump status.
pub const GAMEWATCH_FINAL_JUMP_AERIAL: i32 = 0x1f3;
/// Mr. Game & Watch's Final Smash jumpsquat status.
pub const GAMEWATCH_FINAL_JUMP_SQUAT: i32 = 0x1f1;
/// Mr. Game & Watch's Final Smash landing phase status.
pub const GAMEWATCH_FINAL_LANDING: i32 = 0x1f7;
/// Mr. Game & Watch's Final Smash landing phase light status.
pub const GAMEWATCH_FINAL_LANDING_LIGHT: i32 = 0x1f8;
/// Mr. Game & Watch's Final Smash platform pass-through status.
pub const GAMEWATCH_FINAL_PASS: i32 = 0x1f6;
/// Mr. Game & Watch's Final Smash run phase status.
pub const GAMEWATCH_FINAL_RUN: i32 = 0x1ec;
/// Mr. Game & Watch's Final Smash run phase brake status.
pub const GAMEWATCH_FINAL_RUN_BRAKE: i32 = 0x1ed;
/// Mr. Game & Watch's Final Smash swim dash phase status.
pub const GAMEWATCH_FINAL_SWIM_DASH: i32 = 0x1fd;
/// Mr. Game & Watch's Final Smash swim dive status.
pub const GAMEWATCH_FINAL_SWIM_DIVE: i32 = 0x1f9;
/// Mr. Game & Watch's Final Smash swim jumpsquat status.
pub const GAMEWATCH_FINAL_SWIM_JUMP_SQUAT: i32 = 0x203;
/// Mr. Game & Watch's Final Smash swim rising phase status.
pub const GAMEWATCH_FINAL_SWIM_RISE: i32 = 0x1fa;
/// Mr. Game & Watch's Final Smash swim run phase status.
pub const GAMEWATCH_FINAL_SWIM_RUN: i32 = 0x1fe;
/// Mr. Game & Watch's Final Smash swim braking status.
pub const GAMEWATCH_FINAL_SWIM_RUN_BRAKE: i32 = 0x1ff;
/// Mr. Game & Watch's Final Smash swim turnaround phase status.
pub const GAMEWATCH_FINAL_SWIM_TURN: i32 = 0x200;
/// Mr. Game & Watch's Final Smash swim turn-dash status.
pub const GAMEWATCH_FINAL_SWIM_TURN_DASH: i32 = 0x201;
/// Mr. Game & Watch's Final Smash swim turn-run status.
pub const GAMEWATCH_FINAL_SWIM_TURN_RUN: i32 = 0x202;
/// Mr. Game & Watch's Final Smash swim wait phase status.
pub const GAMEWATCH_FINAL_SWIM_WAIT: i32 = 0x1fb;
/// Mr. Game & Watch's Final Smash swim walk phase status.
pub const GAMEWATCH_FINAL_SWIM_WALK: i32 = 0x1fc;
/// Mr. Game & Watch's Final Smash turnaround phase status.
pub const GAMEWATCH_FINAL_TURN: i32 = 0x1ee;
/// Mr. Game & Watch's Final Smash turn-dash phase status.
pub const GAMEWATCH_FINAL_TURN_DASH: i32 = 0x1ef;
/// Mr. Game & Watch's Final Smash turn-run phase status.
pub const GAMEWATCH_FINAL_TURN_RUN: i32 = 0x1f0;
/// Mr. Game & Watch's Final Smash wait phase status.
pub const GAMEWATCH_FINAL_WAIT: i32 = 0x1e9;
/// Mr. Game & Watch's Final Smash walk phase status.
pub const GAMEWATCH_FINAL_WALK: i32 = 0x1ea;
/// Incineroar's Final Smash dash phase status.
pub const GAOGAEN_FINAL_DASH: i32 = 0x1ed;
/// Incineroar's Final Smash dash ending status.
pub const GAOGAEN_FINAL_DASH_END: i32 = 0x1ee;
/// Incineroar's Final Smash ending status.
pub const GAOGAEN_FINAL_END: i32 = 0x1f3;
/// Incineroar's Final Smash ready phase status.
pub const GAOGAEN_FINAL_READY: i32 = 0x1ef;
/// Incineroar's Final Smash cinematic phase 1 status.
pub const GAOGAEN_FINAL_SCENE01: i32 = 0x1f0;
/// Incineroar's Final Smash cinematic phase 2 status.
pub const GAOGAEN_FINAL_SCENE02: i32 = 0x1f1;
/// Incineroar's Final Smash cinematic phase 3 status.
pub const GAOGAEN_FINAL_SCENE03: i32 = 0x1f2;
/// King K. Rool's Final Smash dash phase status.
pub const KROOL_FINAL_DASH: i32 = 0x1f1;
/// King K. Rool's Final Smash dash ending status.
pub const KROOL_FINAL_DASH_END: i32 = 0x1f2;
/// King K. Rool's Final Smash ending status.
pub const KROOL_FINAL_END: i32 = 0x1f8;
/// King K. Rool's Final Smash ready phase status.
pub const KROOL_FINAL_READY: i32 = 0x1f3;
/// King K. Rool's Final Smash cinematic phase 1 status.
pub const KROOL_FINAL_SCENE01: i32 = 0x1f4;
/// King K. Rool's Final Smash cinematic phase 2 status.
pub const KROOL_FINAL_SCENE02: i32 = 0x1f5;
/// King K. Rool's Final Smash cinematic phase 3 status.
pub const KROOL_FINAL_SCENE03: i32 = 0x1f6;
/// King K. Rool's Final Smash cinematic phase 4 status.
pub const KROOL_FINAL_SCENE04: i32 = 0x1f7;
/// Mii Swordfighter's up special variant 1 ending status.
pub const MIISWORDSMAN_SPECIAL_HI1_END: i32 = 0x1ef;
/// Mii Swordfighter's up special variant 1 jump phase status.
pub const MIISWORDSMAN_SPECIAL_HI1_JUMP: i32 = 0x1ed;
/// Mii Swordfighter's up special variant 1 looping phase status.
pub const MIISWORDSMAN_SPECIAL_HI1_LOOP: i32 = 0x1ee;
/// Mii Swordfighter's up special variant 2 bounce phase status.
pub const MIISWORDSMAN_SPECIAL_HI2_BOUND: i32 = 0x1f2;
/// Mii Swordfighter's up special variant 2 rush phase status.
pub const MIISWORDSMAN_SPECIAL_HI2_RUSH: i32 = 0x1f0;
/// Mii Swordfighter's up special variant 2 rush ending status.
pub const MIISWORDSMAN_SPECIAL_HI2_RUSH_END: i32 = 0x1f1;
/// Mii Swordfighter's up special variant 3 ending status.
pub const MIISWORDSMAN_SPECIAL_HI3_END: i32 = 0x1f4;
/// Mii Swordfighter's up special variant 3 charge hold status.
pub const MIISWORDSMAN_SPECIAL_HI3_HOLD: i32 = 0x1f3;
/// Villager's up special balloon burst status.
pub const MURABITO_SPECIAL_HI_BURST: i32 = 0x1ef;
/// Villager's up special detachment phase status.
pub const MURABITO_SPECIAL_HI_DETACH: i32 = 0x1ee;
/// Villager's up special ending status.
pub const MURABITO_SPECIAL_HI_END: i32 = 0x1ed;
/// Villager's up special balloon-flap phase status.
pub const MURABITO_SPECIAL_HI_FLAP: i32 = 0x1ea;
/// Villager's up special landing phase status.
pub const MURABITO_SPECIAL_HI_LANDING: i32 = 0x1ec;
/// Villager's up special turnaround phase status.
pub const MURABITO_SPECIAL_HI_TURN: i32 = 0x1eb;
/// Villager's up special wait phase status.
pub const MURABITO_SPECIAL_HI_WAIT: i32 = 0x1e9;
/// Pikachu's Final Smash attack phase status.
pub const PIKACHU_FINAL_ATTACK: i32 = 0x1eb;
/// Pikachu's Final Smash second attack status.
pub const PIKACHU_FINAL_ATTACK_2: i32 = 0x1ec;
/// Pikachu's Final Smash finishing attack status.
pub const PIKACHU_FINAL_ATTACK_FINISH: i32 = 0x1ed;
/// Pikachu's Final Smash dash phase status.
pub const PIKACHU_FINAL_DASH: i32 = 0x1e8;
/// Pikachu's Final Smash dash ending status.
pub const PIKACHU_FINAL_DASH_END: i32 = 0x1e9;
/// Pikachu's Final Smash ending status.
pub const PIKACHU_FINAL_END: i32 = 0x1ef;
/// Pikachu's Final Smash hit phase status.
pub const PIKACHU_FINAL_HIT: i32 = 0x1ea;
/// Pikachu's Final Smash return phase status.
pub const PIKACHU_FINAL_RETURN: i32 = 0x1ee;
/// Ice Climbers's up special ledge correction status.
pub const POPO_SPECIAL_HI_CLIFF_COMP: i32 = 0x1e9;
/// Ice Climbers's up special partner ledge correction status.
pub const POPO_SPECIAL_HI_CLIFF_COMP_PARTNER: i32 = 0x1e7;
/// Ice Climbers's up special partner ledge pull status.
pub const POPO_SPECIAL_HI_CLIFF_PULL_PARTNER: i32 = 0x1e8;
/// Ice Climbers's up special failed phase status.
pub const POPO_SPECIAL_HI_FAIL: i32 = 0x1e2;
/// Ice Climbers's up special jump phase status.
pub const POPO_SPECIAL_HI_JUMP: i32 = 0x1e6;
/// Ice Climbers's up special partner jump status.
pub const POPO_SPECIAL_HI_JUMP_PARTNER: i32 = 0x1e5;
/// Ice Climbers's up special pre-jump phase status.
pub const POPO_SPECIAL_HI_JUMP_PRE: i32 = 0x1e3;
/// Ice Climbers's up special partner phase status.
pub const POPO_SPECIAL_HI_PARTNER: i32 = 0x1e4;
/// Robin's Final Smash attack phase status.
pub const REFLET_FINAL_ATTACK: i32 = 0x1fc;
/// Robin's Final Smash chrom attack phase status.
pub const REFLET_FINAL_CHROM_ATTACK: i32 = 0x1f6;
/// Robin's Final Smash chrom dash phase status.
pub const REFLET_FINAL_CHROM_DASH: i32 = 0x1f1;
/// Robin's Final Smash chrom entry phase status.
pub const REFLET_FINAL_CHROM_ENTRY: i32 = 0x1f0;
/// Robin's Final Smash chrom exit phase status.
pub const REFLET_FINAL_CHROM_EXIT: i32 = 0x1f7;
/// Robin's Final Smash chrom launch phase status.
pub const REFLET_FINAL_CHROM_LAUNCH: i32 = 0x1f2;
/// Robin's Final Smash chrom miss phase status.
pub const REFLET_FINAL_CHROM_MISS: i32 = 0x1f3;
/// Robin's Final Smash chrom ready phase status.
pub const REFLET_FINAL_CHROM_READY: i32 = 0x1f5;
/// Robin's Final Smash chrom rush phase status.
pub const REFLET_FINAL_CHROM_RUSH: i32 = 0x1f4;
/// Robin's Final Smash ending status.
pub const REFLET_FINAL_END: i32 = 0x1ef;
/// Robin's Final Smash failed phase status.
pub const REFLET_FINAL_FAIL: i32 = 0x1ed;
/// Robin's Final Smash hit phase status.
pub const REFLET_FINAL_HIT: i32 = 0x1fa;
/// Robin's Final Smash movement phase status.
pub const REFLET_FINAL_MOVE: i32 = 0x1fb;
/// Robin's Final Smash ready phase status.
pub const REFLET_FINAL_READY: i32 = 0x1ee;
/// Robin's Final Smash startup status.
pub const REFLET_FINAL_START: i32 = 0x1ec;
/// Ridley's Final Smash dash phase status.
pub const RIDLEY_FINAL_DASH: i32 = 0x1f9;
/// Ridley's Final Smash dash ending status.
pub const RIDLEY_FINAL_DASH_END: i32 = 0x1fa;
/// Ridley's Final Smash ending status.
pub const RIDLEY_FINAL_END: i32 = 0x201;
/// Ridley's Final Smash ready phase status.
pub const RIDLEY_FINAL_READY: i32 = 0x1fb;
/// Ridley's Final Smash cinematic phase 1 status.
pub const RIDLEY_FINAL_SCENE01: i32 = 0x1fc;
/// Ridley's Final Smash cinematic phase 2 status.
pub const RIDLEY_FINAL_SCENE02: i32 = 0x1fd;
/// Ridley's Final Smash cinematic phase 3 status.
pub const RIDLEY_FINAL_SCENE03: i32 = 0x1fe;
/// Ridley's Final Smash cinematic phase 4 status.
pub const RIDLEY_FINAL_SCENE04: i32 = 0x1ff;
/// Ridley's Final Smash cinematic phase 5 status.
pub const RIDLEY_FINAL_SCENE05: i32 = 0x200;
/// Ridley's up special charge phase f status.
pub const RIDLEY_SPECIAL_HI_CHARGE_F: i32 = 0x1f0;
/// Ridley's up special charge phase hi status.
pub const RIDLEY_SPECIAL_HI_CHARGE_HI: i32 = 0x1f2;
/// Ridley's up special charge startup status.
pub const RIDLEY_SPECIAL_HI_CHARGE_START: i32 = 0x1ef;
/// Ridley's up special ending status.
pub const RIDLEY_SPECIAL_HI_END: i32 = 0x1f7;
/// Ridley's up special hover phase status.
pub const RIDLEY_SPECIAL_HI_HOVER: i32 = 0x1ee;
/// Ridley's up special landing phase status.
pub const RIDLEY_SPECIAL_HI_LANDING: i32 = 0x1f6;
/// Ridley's up special ceiling collision status.
pub const RIDLEY_SPECIAL_HI_STOP_CEIL: i32 = 0x1f4;
/// Ridley's up special wall collision status.
pub const RIDLEY_SPECIAL_HI_STOP_WALL: i32 = 0x1f5;
/// Ryu's second Final Smash status.
pub const RYU_FINAL2: i32 = 0x1fe;
/// Ryu's second Final Smash aerial ending status.
pub const RYU_FINAL2_AIR_END: i32 = 0x201;
/// Ryu's second Final Smash fall phase status.
pub const RYU_FINAL2_FALL: i32 = 0x1ff;
/// Ryu's second Final Smash landing phase status.
pub const RYU_FINAL2_LANDING: i32 = 0x200;
/// Ryu's Final Smash aerial ending status.
pub const RYU_FINAL_AIR_END: i32 = 0x1fc;
/// Ryu's Final Smash fall phase status.
pub const RYU_FINAL_FALL: i32 = 0x1fa;
/// Ryu's Final Smash hit phase status.
pub const RYU_FINAL_HIT: i32 = 0x1fd;
/// Ryu's Final Smash jump phase status.
pub const RYU_FINAL_JUMP: i32 = 0x1f9;
/// Ryu's Final Smash landing phase status.
pub const RYU_FINAL_LANDING: i32 = 0x1fb;
/// Samus's Final Smash ending airborne status.
pub const SAMUS_FINAL_END_A: i32 = 0x1f5;
/// Samus's Final Smash ending grounded status.
pub const SAMUS_FINAL_END_G: i32 = 0x1f4;
/// Samus's Final Smash looping phase airborne status.
pub const SAMUS_FINAL_LOOP_A: i32 = 0x1f3;
/// Samus's Final Smash looping phase grounded status.
pub const SAMUS_FINAL_LOOP_G: i32 = 0x1f2;
/// Samus's Final Smash startup airborne status.
pub const SAMUS_FINAL_START_A: i32 = 0x1f1;
/// Samus's Final Smash startup grounded status.
pub const SAMUS_FINAL_START_G: i32 = 0x1f0;
/// Samus's Final Smash transformation into Zero Suit Samus status.
pub const SAMUS_FINAL_SZEROSUIT: i32 = 0x1ef;
/// Sheik's Final Smash attack phase status.
pub const SHEIK_FINAL_ATTACK: i32 = 0x1f0;
/// Sheik's Final Smash dash phase status.
pub const SHEIK_FINAL_DASH: i32 = 0x1ed;
/// Sheik's Final Smash dash ending status.
pub const SHEIK_FINAL_DASH_END: i32 = 0x1ee;
/// Sheik's Final Smash ending status.
pub const SHEIK_FINAL_END: i32 = 0x1f2;
/// Sheik's Final Smash finisher status.
pub const SHEIK_FINAL_FINISH: i32 = 0x1f1;
/// Sheik's Final Smash hit phase status.
pub const SHEIK_FINAL_HIT: i32 = 0x1ef;
/// Sheik's Final Smash startup status.
pub const SHEIK_FINAL_START: i32 = 0x1ec;
/// Diddy Kong's up special charge damage reaction status.
pub const DIDDY_SPECIAL_HI_CHARGE_DAMAGE: i32 = 0x1f1;
/// Diddy Kong's up special Rocketbarrel damage reaction status.
pub const DIDDY_SPECIAL_HI_UPPER_DAMAGE: i32 = 0x1f3;
/// Diddy Kong's side special damage fall status.
pub const DIDDY_SPECIAL_S_DAMAGE_FALL: i32 = 0x1ef;
/// Greninja's up special wall-impact damage reaction status.
pub const GEKKOUGA_SPECIAL_HI_WALL_DAMAGE: i32 = 0x1e8;
/// Mr. Game & Watch's down special wait startup status.
pub const GAMEWATCH_SPECIAL_LW_WAIT_START: i32 = 0x1e3;
/// Mr. Game & Watch's down special wait phase status.
pub const GAMEWATCH_SPECIAL_LW_WAIT: i32 = 0x1e4;
/// Mr. Game & Watch's down special capture phase status.
pub const GAMEWATCH_SPECIAL_LW_CATCH: i32 = 0x1e5;
/// Mr. Game & Watch's down special reflect phase status.
pub const GAMEWATCH_SPECIAL_LW_REFLECT: i32 = 0x1e6;
/// Mr. Game & Watch's down special release phase status.
pub const GAMEWATCH_SPECIAL_LW_SHOOT: i32 = 0x1e7;
/// Mr. Game & Watch's down special ending status.
pub const GAMEWATCH_SPECIAL_LW_END: i32 = 0x1e8;
/// Ganondorf's side special capture phase status.
pub const GANON_SPECIAL_S_CATCH: i32 = 0x1e2;
/// Ganondorf's aerial side special capture phase status.
#[cfg(test)]
pub const GANON_SPECIAL_AIR_S_CATCH: i32 = 0x1e3;
/// Bowser Jr.'s up special damage ending status.
pub const KOOPAJR_SPECIAL_HI_DAMAGE_END: i32 = 0x1eb;
/// Squirtle's down special swap-out phase status.
pub const PZENIGAME_SPECIAL_LW_OUT: i32 = 0x1e2;
/// Squirtle's down special inactive standby status.
pub const PZENIGAME_SPECIAL_LW_STANDBY: i32 = 0x1e1;
/// Ivysaur's down special swap-out phase status.
pub const PFUSHIGISOU_SPECIAL_LW_OUT: i32 = 0x1e2;
/// Ivysaur's down special inactive standby status.
pub const PFUSHIGISOU_SPECIAL_LW_STANDBY: i32 = 0x1e1;
/// Charizard's down special swap-out phase status.
pub const PLIZARDON_SPECIAL_LW_OUT: i32 = 0x1e2;
/// Charizard's down special inactive standby status.
pub const PLIZARDON_SPECIAL_LW_STANDBY: i32 = 0x1e1;
/// Pyra's down special swap-out phase status.
pub const EFLAME_SPECIAL_LW_OUT: i32 = 0x1e2;
/// Pyra's down special inactive standby status.
pub const EFLAME_SPECIAL_LW_STANDBY: i32 = 0x1e1;
/// Mythra's down special swap-out phase status.
pub const ELIGHT_SPECIAL_LW_OUT: i32 = 0x1e2;
/// Mythra's down special inactive standby status.
pub const ELIGHT_SPECIAL_LW_STANDBY: i32 = 0x1e1;
// In the native table these two endpoints enclose Kirby's inhale animation states.
/// Kirby's neutral special looping phase status.
pub const KIRBY_SPECIAL_N_LOOP: i32 = 0x1e1;
/// Kirby's neutral special spit status.
pub const KIRBY_SPECIAL_N_SPIT: i32 = 0x1f2;
/// Bowser's down special grounded status.
pub const KOOPA_SPECIAL_LW_G: i32 = 0x1e3;
