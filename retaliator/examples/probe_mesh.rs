//! Lane H2 probe -- the capybara center-mesh opening (lane-g H1): (a) does forcing the
//! mesh prefix suppress Great Barrier's 26-move wall book mega-loop, and (b) does it
//! hold or improve OUR margins vs v1/v2/scoutbase? PROBE-ONLY: no `retaliator/src/*`
//! changes; forced prefixes + shipped searches only.
//!
//! Forcing pattern (extends lane-h `probe_bluefirst`, catalog-0 fix, to multi-move
//! prefixes): a forced opening plays as engine action 1 (the site consumed that side's
//! first own move; the prefix is simply delayed one tempo, entries are never skipped --
//! the probe_mesh v1 off-by-one lesson). A bot with a prefix plays those moves as its own
//! moves, in order; the first prefix move that is ILLEGAL at its turn TRUNCATES the prefix
//! (fall-throughs are counted and reported, never silent -- the probe_bluechair no-op
//! lesson). After the prefix everyone plays their shipped search:
//! us = `retaliator::search::best_move` (v3); the GB clone plays scoutbase (its described
//! engine family) or v3 as a second configuration.
//!
//! Prefixes (parsed with the engine's own `notation::parse_square`; ids printed+asserted
//! at startup against the research-file ids):
//!   MESH-B (capy as Blue, lane-g #4): D10-D13 D10-G8 G8-J10 J10-G13 G8-J11 G13-J11
//!                                    G13-D10 J11-M13  (mesh6 = first 6, mesh8 = all 8)
//!   MESH-R (capy as Red, mirror):    P10-M8 M8-J10 J10-M13 M8-J11 M13-J11 M13-P11
//!                                    M13-P10 P11-M8
//!   GB-B book (rival-analysis #4; GB own-moves 2..13 after the site-forced action-1):
//!     D10-E12 D10-F13 E12-G15 F13-H16 G15-I18 H16-J19 I18-J19 D10-F7 D10-E8 E8-G5 F7-H4 G5-I2
//!   GB-R book (GB red own-moves 1..13, the exact mirror + L4-J1):
//!     P10-O12 P10-N13 O12-M15 N13-L16 M15-K18 L16-J19 K18-J19 P10-N7 P10-O8 O8-M5 N7-L4 M5-K2 L4-J1
//!
//! Numbering: engine actions 1..120 (B[1] solo, R[2,3], B[4,5], ...). Close gains are
//! live-area deltas across the Connect move (lane-g's corrected metric). mega[34-42] =
//! a side's largest close gain at engine acts 34..=42 (the act-38-40 mega-loop window).
//! Lane-g H1 target: with the mesh on, GB largest loop <= ~30 and mega < 100; unsuppressed
//! GB = largest 51.5+, mega > 100.
//!
//! Sections (CLI arg): smoke | a-scout | a-v3 | b-v1 | b-v2 | b-scout | all.
//! Part (a): GB-Red vs us-Blue on the 5 standard openings (none = blue_opener D10-F7 for
//! base); GB-Blue on its 3 observed site action-1s (rival-analysis #4), us Red. n=5/3
//! openings x 3 treatments per GB engine.
//! Part (b): 5 openings x 2 colors x 3 treatments per opponent (n=10 per claim; the 4
//! site-forced openings alone give n=8, 4 x 2 colors). The b-v1 base rows must reproduce
//! probe_v3all byte-for-byte (the validation gate).

#[path = "support/v1base.rs"]
mod v1base;
#[path = "support/v2base.rs"]
mod v2base;
#[path = "support/scoutbase.rs"]
mod scoutbase;

use meridian_engine::{notation, Game, Move, MoveKind, Player, Position};

type SearchFn = fn(&Position) -> Option<Move>;

const MESH_B: &str = "D10-D13 D10-G8 G8-J10 J10-G13 G8-J11 G13-J11 G13-D10 J11-M13";
const MESH_R: &str = "P10-M8 M8-J10 J10-M13 M8-J11 M13-J11 M13-P11 M13-P10 P11-M8";
const GB_BOOK_B: &str =
    "D10-E12 D10-F13 E12-G15 F13-H16 G15-I18 H16-J19 I18-J19 D10-F7 D10-E8 E8-G5 F7-H4 G5-I2";
const GB_BOOK_R: &str =
    "P10-O12 P10-N13 O12-M15 N13-L16 M15-K18 L16-J19 K18-J19 P10-N7 P10-O8 O8-M5 N7-L4 M5-K2 L4-J1";

/// Engine-action checkpoints at which both sides' live area and score are recorded.
const CKPT: [u8; 6] = [11, 26, 30, 38, 40, 50];

fn v3(pos: &Position) -> Option<Move> {
    retaliator::search::best_move(pos)
}

fn parse_line(text: &str) -> Vec<Move> {
    text.split_whitespace()
        .map(|t| {
            let (a, b) = t.split_once('-').expect("move is SRC-TGT");
            Move::between(
                notation::parse_square(a).expect("source square"),
                notation::parse_square(b).expect("target square"),
            )
            .expect("1-3 king steps")
        })
        .collect()
}

fn one(text: &str) -> Move {
    let v = parse_line(text);
    assert_eq!(v.len(), 1, "one() wants a single move");
    v[0]
}

fn move_txt(mv: Move) -> String {
    format!(
        "{}-{}",
        notation::square(mv.source),
        notation::square(mv.target().expect("on board"))
    )
}

/// A player: an optional forced prefix (own moves, in order) + a search hand-off.
struct Bot {
    prefix: Vec<Move>,
    search: SearchFn,
    /// Prefix entries already played. Independent of how many own moves the site consumed:
    /// entry 0 is ALWAYS the next prefix move to play (a forced opening just delays the
    /// prefix by one tempo; it never skips entries -- the probe_mesh v1 off-by-one lesson).
    forced_idx: usize,
    expected: usize,
    inj: usize,
    /// First prefix slot that could not be played: (slot, engine act, why).
    stall: Option<(usize, u8, String)>,
}

impl Bot {
    fn free(search: SearchFn) -> Bot {
        Bot { prefix: Vec::new(), search, forced_idx: 0, expected: 0, inj: 0, stall: None }
    }
    fn forced(prefix: Vec<Move>, search: SearchFn) -> Bot {
        let expected = prefix.len();
        Bot { prefix, search, forced_idx: 0, expected, inj: 0, stall: None }
    }
    fn next(&mut self, pos: &Position, act1: u8) -> Move {
        if self.forced_idx < self.prefix.len() {
            let mv = self.prefix[self.forced_idx];
            match pos.check_move(mv) {
                Ok(_) => {
                    self.forced_idx += 1;
                    self.inj += 1;
                    return mv;
                }
                Err(why) => {
                    if self.stall.is_none() {
                        self.stall = Some((self.forced_idx, act1, format!("{why}")));
                    }
                    // Truncate: the prefix is dead, search takes over for good.
                    self.forced_idx = self.prefix.len();
                }
            }
        }
        (self.search)(pos).expect("search returns a move")
    }
}

struct Row {
    score: [f64; 2],
    area: [f64; 2],
    /// (engine act, live-area gain) per Connect move, [Blue, Red].
    closes: [Vec<(u8, f64)>; 2],
    /// (engine act, [area B, R], [score B, R]) at each CKPT action.
    traj: Vec<(u8, [f64; 2], [f64; 2])>,
    /// (injected, expected) per side.
    inj: [(usize, usize); 2],
    stall: [Option<(usize, u8, String)>; 2],
}

impl Row {
    fn new() -> Row {
        Row {
            score: [0.0; 2],
            area: [0.0; 2],
            closes: [Vec::new(), Vec::new()],
            traj: Vec::new(),
            inj: [(0, 0); 2],
            stall: [None, None],
        }
    }
    fn largest(&self, idx: usize) -> (f64, u8) {
        self.closes[idx]
            .iter()
            .map(|&(a, g)| (g, a))
            .fold((0.0, 0), |best, x| if x.0 > best.0 { x } else { best })
    }
    /// Largest close gain at engine acts 34..=42 (the act-38-40 mega-loop window).
    fn mega(&self, idx: usize) -> f64 {
        self.closes[idx]
            .iter()
            .filter(|(a, _)| (34..=42).contains(a))
            .map(|(_, g)| *g)
            .fold(0.0, f64::max)
    }
    /// (live area, banked score) at a checkpoint act, for side idx.
    fn at(&self, act: u8, idx: usize) -> Option<(f64, f64)> {
        self.traj
            .iter()
            .find(|(a, _, _)| *a == act)
            .map(|(_, ar, sc)| (ar[idx], sc[idx]))
    }
}

fn play(opening: Option<Move>, mut blue: Bot, mut red: Bot) -> Row {
    let mut game = Game::new();
    if let Some(mv) = opening {
        game.play(mv).expect("legal opening");
    }
    let mut row = Row::new();
    while !game.is_over() {
        let pos = game.position();
        let mover = pos.to_move();
        let act1 = pos.actions_played() + 1;
        let mv = match mover {
            Player::Blue => blue.next(&pos, act1),
            Player::Red => red.next(&pos, act1),
        };
        let kind = pos.check_move(mv).ok();
        let before = pos.area(mover).to_f64();
        game.play(mv).expect("legal move");
        let after = game.position();
        let ap = after.actions_played();
        if kind == Some(MoveKind::Connect) {
            let gain = after.area(mover).to_f64() - before;
            row.closes[mover as usize].push((ap, gain));
        }
        if CKPT.contains(&ap) {
            row.traj.push((
                ap,
                [after.area(Player::Blue).to_f64(), after.area(Player::Red).to_f64()],
                [after.score(Player::Blue).to_f64(), after.score(Player::Red).to_f64()],
            ));
        }
    }
    let fin = game.position();
    row.score = [fin.score(Player::Blue).to_f64(), fin.score(Player::Red).to_f64()];
    row.area = [fin.area(Player::Blue).to_f64(), fin.area(Player::Red).to_f64()];
    row.inj = [(blue.inj, blue.expected), (red.inj, red.expected)];
    row.stall = [blue.stall, red.stall];
    row
}

/// One game record for the summaries.
struct Rec {
    part: &'static str,
    key: String,
    open: String,
    side: &'static str,
    treat: &'static str,
    us_idx: usize,
    row: Row,
}

fn margin(rec: &Rec) -> f64 {
    let (u, t) = (rec.row.score[rec.us_idx], rec.row.score[1 - rec.us_idx]);
    (u - t) / u.max(1.0) * 100.0
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len().max(1) as f64
}

fn stall_txt(row: &Row, idx: usize, tag: &str) -> String {
    match &row.stall[idx] {
        None => String::new(),
        Some((slot, act, why)) => format!(" STALL[{tag}#{slot}@act{act}:{why}]"),
    }
}

fn us_bot(treat: &str, full: &[Move], search: SearchFn) -> Bot {
    match treat {
        "base" => Bot::free(search),
        "mesh6" => Bot::forced(full[..6].to_vec(), search),
        "mesh8" => Bot::forced(full.to_vec(), search),
        _ => panic!("unknown treatment {treat}"),
    }
}

fn std_opens() -> Vec<(&'static str, Option<Move>)> {
    vec![
        ("none", None),
        ("4864", Some(Move::from_index(4864).expect("4864"))),
        ("5589", Some(Move::from_index(5589).expect("5589"))),
        ("11723", Some(Move::from_index(11723).expect("11723"))),
        ("9199", Some(Move::from_index(9199).expect("9199"))),
    ]
}

fn part_a(
    gb_name: &str,
    gb: SearchFn,
    mesh_b: &[Move],
    mesh_r: &[Move],
    gbook_b: &[Move],
    gbook_r: &[Move],
    recs: &mut Vec<Rec>,
) {
    let treatments = ["base", "mesh6", "mesh8"];
    // GB as Red: its full 13-move book, no site interference. The site forces OUR action-1
    // (we are Blue): the 5 standard openings (none = blue_opener D10-F7 for the base).
    for (oname, open) in std_opens() {
        for treat in treatments {
            let us = us_bot(treat, mesh_b, v3);
            let gbot = Bot::forced(gbook_r.to_vec(), gb);
            let row = play(open, us, gbot);
            recs.push(Rec {
                part: "a",
                key: format!("GB=red ENG={gb_name}"),
                open: oname.to_string(),
                side: "blue",
                treat,
                us_idx: 0,
                row,
            });
        }
    }
    // GB as Blue: the site forces its action-1 (rival-analysis #4: D10-F11 / D10-C7 /
    // A10-C11 observed), then the 12-move book. We are Red with the mirrored mesh.
    let gb_opens = [("D10-F11", one("D10-F11")), ("D10-C7", one("D10-C7")), ("A10-C11", one("A10-C11"))];
    for (oname, open) in gb_opens {
        for treat in treatments {
            let us = us_bot(treat, mesh_r, v3);
            let gbot = Bot::forced(gbook_b.to_vec(), gb);
            let row = play(Some(open), gbot, us);
            recs.push(Rec {
                part: "a",
                key: format!("GB=blue ENG={gb_name}"),
                open: oname.to_string(),
                side: "red",
                treat,
                us_idx: 1,
                row,
            });
        }
    }
}

fn part_b(vs_name: &str, vs: SearchFn, mesh_b: &[Move], mesh_r: &[Move], recs: &mut Vec<Rec>) {
    for (oname, open) in std_opens() {
        for us_blue in [true, false] {
            for treat in ["base", "mesh6", "mesh8"] {
                let them = Bot::free(vs);
                let row = if us_blue {
                    let us = us_bot(treat, mesh_b, v3);
                    play(open, us, them)
                } else {
                    let us = us_bot(treat, mesh_r, v3);
                    play(open, them, us)
                };
                recs.push(Rec {
                    part: "b",
                    key: format!("VS={vs_name}"),
                    open: oname.to_string(),
                    side: if us_blue { "blue" } else { "red" },
                    treat,
                    us_idx: usize::from(!us_blue),
                    row,
                });
            }
        }
    }
}

fn report_a(rec: &Rec) {
    let row = &rec.row;
    let (ui, gi) = (rec.us_idx, 1 - rec.us_idx);
    let (us, them) = (row.score[ui], row.score[gi]);
    let (gl, gla) = row.largest(gi);
    let (ul, ula) = row.largest(ui);
    let a40 = row.at(40, gi);
    let a38 = row.at(38, gi);
    let gfirst = row.closes[gi].first().map(|(a, _)| *a).unwrap_or(0);
    let ufirst = row.closes[ui].first().map(|(a, _)| *a).unwrap_or(0);
    println!(
        "A {key} OPEN={open} TREAT={treat} FINAL={us:.0}-{them:.0} M={m:+.1}% \
GB:1st={gfirst} larg={gl:.1}@{gla} mega={mega:.1} a38={a38:.1} a40={a40v:.1} s40={s40:.0} finA={fa:.1} cls={cls} gbinj={gbinj}{gbstall} \
US:1st={ufirst} larg={ul:.1}@{ula} inj={inj}{stall}",
        key = rec.key,
        open = rec.open,
        treat = rec.treat,
        us = us,
        them = them,
        m = margin(rec),
        gfirst = gfirst,
        gl = gl,
        gla = gla,
        mega = row.mega(gi),
        a38 = a38.map(|(a, _)| a).unwrap_or(-1.0),
        a40v = a40.map(|(a, _)| a).unwrap_or(-1.0),
        s40 = a40.map(|(_, s)| s).unwrap_or(-1.0),
        fa = row.area[gi],
        cls = row.closes[gi].len(),
        ufirst = ufirst,
        ul = ul,
        ula = ula,
        inj = format!("{}/{}", row.inj[ui].0, row.inj[ui].1),
        stall = stall_txt(row, ui, "us"),
        gbinj = format!("{}/{}", row.inj[gi].0, row.inj[gi].1),
        gbstall = stall_txt(row, gi, "gb"),
    );
}

fn report_b(rec: &Rec) {
    let row = &rec.row;
    let (ui, ti) = (rec.us_idx, 1 - rec.us_idx);
    let (us, them) = (row.score[ui], row.score[ti]);
    let (ul, ula) = row.largest(ui);
    let ufirst = row.closes[ui].first().map(|(a, _)| *a).unwrap_or(0);
    let a11 = row.at(11, ui).map(|(a, _)| a).unwrap_or(-1.0);
    println!(
        "B {key} OPEN={open} SIDE={side} TREAT={treat} FINAL={us:.0}-{them:.0} M={m:+.1}% \
Afin={au:.1}/{at:.1} US:1st={ufirst} larg={ul:.1}@{ula} a11={a11:.1} inj={inj}{stall}",
        key = rec.key,
        open = rec.open,
        side = rec.side,
        treat = rec.treat,
        us = us,
        them = them,
        m = margin(rec),
        au = row.area[ui],
        at = row.area[ti],
        ufirst = ufirst,
        ul = ul,
        ula = ula,
        a11 = a11,
        inj = format!("{}/{}", row.inj[ui].0, row.inj[ui].1),
        stall = stall_txt(row, ui, "us"),
    );
}

fn distinct_keys(recs: &[Rec], part: &str) -> Vec<String> {
    let mut keys: Vec<String> = Vec::new();
    for r in recs {
        if r.part == part && !keys.contains(&r.key) {
            keys.push(r.key.clone());
        }
    }
    keys
}

fn summaries(recs: &[Rec]) {
    let a_keys = distinct_keys(recs, "a");
    if !a_keys.is_empty() {
        println!("== A SUMMARY: GB book+search vs us (treatments; GB metrics are the suppression readout) ==");
        for key in &a_keys {
            for treat in ["base", "mesh6", "mesh8"] {
                let rs: Vec<&Rec> =
                    recs.iter().filter(|r| r.part == "a" && r.key == *key && r.treat == treat).collect();
                if rs.is_empty() {
                    continue;
                }
                let n = rs.len();
                let wins = rs.iter().filter(|r| r.row.score[r.us_idx] > r.row.score[1 - r.us_idx]).count();
                let gi = 1 - rs[0].us_idx;
                let larg: Vec<f64> = rs.iter().map(|r| r.row.largest(gi).0).collect();
                let mega: Vec<f64> = rs.iter().map(|r| r.row.mega(gi)).collect();
                let a40: Vec<f64> = rs.iter().map(|r| r.row.at(40, gi).map(|(a, _)| a).unwrap_or(0.0)).collect();
                let s40: Vec<f64> = rs.iter().map(|r| r.row.at(40, gi).map(|(_, s)| s).unwrap_or(0.0)).collect();
                println!(
                    "{key} TREAT={treat} n={n} usW-L={wins}-{lw} meanM={mm:+.1}% \
GBlarg mean/max={lg:.1}/{lgx:.1} GBmega mean/max={mg:.1}/{mgx:.1} GBa@40={a40:.1} GBs@40={s40:.0}",
                    lw = n - wins,
                    mm = rs.iter().map(|r| margin(r)).sum::<f64>() / n as f64,
                    lg = mean(&larg),
                    lgx = larg.iter().cloned().fold(f64::MIN, f64::max),
                    mg = mean(&mega),
                    mgx = mega.iter().cloned().fold(f64::MIN, f64::max),
                    a40 = mean(&a40),
                    s40 = mean(&s40),
                );
            }
        }
    }
    let b_keys = distinct_keys(recs, "b");
    if !b_keys.is_empty() {
        println!("== B SUMMARY per (opponent, our side, treatment); base rows must reproduce probe_v3all ==");
        for key in &b_keys {
            for side in ["blue", "red"] {
                for treat in ["base", "mesh6", "mesh8"] {
                    let rs: Vec<&Rec> = recs
                        .iter()
                        .filter(|r| r.part == "b" && r.key == *key && r.side == side && r.treat == treat)
                        .collect();
                    if rs.is_empty() {
                        continue;
                    }
                    let n = rs.len();
                    let wins = rs.iter().filter(|r| r.row.score[r.us_idx] > r.row.score[1 - r.us_idx]).count();
                    let ms: Vec<f64> = rs.iter().map(|r| margin(r)).collect();
                    println!(
                        "{key} SIDE={side} TREAT={treat} n={n} W-L={wins}-{lw} meanM={mm:+.1}% worst={w:+.1}%",
                        lw = n - wins,
                        mm = mean(&ms),
                        w = ms.iter().cloned().fold(f64::INFINITY, f64::min)
                    );
                }
            }
        }
        println!("== B FORCED-OPENINGS-ONLY (the site-forced starts; n=8 = 4 opens x 2 colors) ==");
        for key in &b_keys {
            for treat in ["base", "mesh6", "mesh8"] {
                let rs: Vec<&Rec> = recs
                    .iter()
                    .filter(|r| r.part == "b" && r.key == *key && r.treat == treat && r.open != "none")
                    .collect();
                if rs.is_empty() {
                    continue;
                }
                let n = rs.len();
                let wins = rs.iter().filter(|r| r.row.score[r.us_idx] > r.row.score[1 - r.us_idx]).count();
                println!(
                    "{key} TREAT={treat} n={n} W-L={wins}-{lw} meanM={mm:+.1}%",
                    lw = n - wins,
                    mm = rs.iter().map(|r| margin(r)).sum::<f64>() / n as f64
                );
            }
        }
        println!("== B per-opening spread (our margin: base | mesh6 | mesh8) ==");
        for key in &b_keys {
            for open in ["none", "4864", "5589", "11723", "9199"] {
                for side in ["blue", "red"] {
                    let mut cells = [String::new(), String::new(), String::new()];
                    for (i, treat) in ["base", "mesh6", "mesh8"].iter().enumerate() {
                        if let Some(r) = recs.iter().find(|r| {
                            r.part == "b" && r.key == *key && r.open == open && r.side == side && r.treat == *treat
                        }) {
                            cells[i] = format!("{:+.1}{}", margin(r), if r.row.score[r.us_idx] > r.row.score[1 - r.us_idx] { "W" } else { "L" });
                        }
                    }
                    println!("{key} OPEN={open} SIDE={side}: {} | {} | {}", cells[0], cells[1], cells[2]);
                }
            }
        }
    }
}

fn main() {
    // Id decode gate (rival-analysis #2: D10-D13 = 16058; catalog/lane-h: D10-F7 = 1979,
    // A10-C11 = 11723): a mismatch here means the notation parse is wrong, abort.
    assert_eq!(one("D10-D13").index(), 16058, "D10-D13 must decode to 16058");
    assert_eq!(one("D10-F7").index(), 1979, "D10-F7 must decode to 1979");
    assert_eq!(one("A10-C11").index(), 11723, "A10-C11 must decode to 11723");

    let mesh_b = parse_line(MESH_B);
    let mesh_r = parse_line(MESH_R);
    let gbook_b = parse_line(GB_BOOK_B);
    let gbook_r = parse_line(GB_BOOK_R);

    for (name, list) in [("MESH-B", &mesh_b), ("MESH-R", &mesh_r), ("GB-B", &gbook_b), ("GB-R", &gbook_r)] {
        let ids = list.iter().map(|m| m.index().to_string()).collect::<Vec<_>>().join(",");
        let txt = list.iter().map(|m| move_txt(*m)).collect::<Vec<_>>().join(" ");
        println!("{name}: {txt}  ids=[{ids}]");
    }
    let gf11 = one("D10-F11");
    let gc7 = one("D10-C7");
    println!("GB blue action-1s: D10-F11={} D10-C7={}", gf11.index(), gc7.index());

    let section = std::env::args().nth(1).unwrap_or_else(|| "all".to_string());
    let mut recs: Vec<Rec> = Vec::new();
    match section.as_str() {
        "smoke" => {
            // One game, fast: GB-Red book (scoutbase post) vs mesh6-Blue, no opening.
            let us = us_bot("mesh6", &mesh_b, v3);
            let gbot = Bot::forced(gbook_r.to_vec(), scoutbase::best_move);
            let row = play(None, us, gbot);
            recs.push(Rec {
                part: "a",
                key: "GB=red ENG=scoutbase".to_string(),
                open: "none".to_string(),
                side: "blue",
                treat: "mesh6",
                us_idx: 0,
                row,
            });
        }
        "a-scout" => part_a("scoutbase", scoutbase::best_move, &mesh_b, &mesh_r, &gbook_b, &gbook_r, &mut recs),
        "a-v3" => part_a("v3", v3, &mesh_b, &mesh_r, &gbook_b, &gbook_r, &mut recs),
        "b-v1" => part_b("v1", v1base::best_move, &mesh_b, &mesh_r, &mut recs),
        "b-v2" => part_b("v2", v2base::best_move, &mesh_b, &mesh_r, &mut recs),
        "b-scout" => part_b("scout", scoutbase::best_move, &mesh_b, &mesh_r, &mut recs),
        "all" => {
            part_a("scoutbase", scoutbase::best_move, &mesh_b, &mesh_r, &gbook_b, &gbook_r, &mut recs);
            part_a("v3", v3, &mesh_b, &mesh_r, &gbook_b, &gbook_r, &mut recs);
            part_b("v1", v1base::best_move, &mesh_b, &mesh_r, &mut recs);
            part_b("v2", v2base::best_move, &mesh_b, &mesh_r, &mut recs);
            part_b("scout", scoutbase::best_move, &mesh_b, &mesh_r, &mut recs);
        }
        other => panic!("unknown section {other}"),
    }
    for r in &recs {
        if r.part == "a" {
            report_a(r);
        } else {
            report_b(r);
        }
    }
    summaries(&recs);
}
