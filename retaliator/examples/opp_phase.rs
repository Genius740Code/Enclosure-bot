//! Lane O v7 PhaseTracker MEASUREMENT probe (opp_phase.rs).
//!
//! Implements `PhaseTracker` from `research/v7-phase-spec.md` as a
//! harness-side measurement ONLY: turn-end snapshots + the 5 proposed phases
//! (BUILD/CONTEST/WALL-RACE/BANK-RACE/DENY) with the proposed rate triggers.
//! No eval/search changes. Deterministic, engine legality only.
//!
//! Foe lines (us = retaliator `search::best_move` throughout):
//!   A "collapser" = scoutbase rush-closer (CONTEST positive control)
//!   B "gbstyle"   = wall-builder, closes late big (WALL-RACE positive control)
//!   C "v1base"    = normal Scout-shaped foe (quiet line, FP control)
//! NOTE: the task named a "capybara" line; no capybara mimic exists in-tree
//! (grep over retaliator/bots/harness/vendor finds nothing), so v1base stands
//! in as the quiet third line. This substitution is recorded in
//! `research/o-phase-validation.md`.
//!
//! Grid per line: skip {0,10,20,30} x us-color {blue,red} = 8 games (n>=8).
//! Metrics per game: phase-dwell turns, transition list, big-close events
//! (foe Connect with area gain >= 8) with phase lead time, FP views.

#[path = "support/scoutbase.rs"]
mod scoutbase;
#[path = "support/v1base.rs"]
mod v1base;
mod opp_collapser;
mod opp_gbstyle;

use meridian_engine::{Game, Move, MoveKind, Player, Position};

/// Proposed trigger constants (v7-phase-spec.md, PROPOSED revision P0).
/// P1 retune (single allowed revision, driven by P0diag 24-game rates):
/// - CONTEST_ENTER_RATE 0.75 -> 0.5: rush-line early peak was 0.5-0.67 in
///   8/8 collapser games, reaching 0.75 early in 0/8.
/// - WALL_ENTER_BUILD 3.0 -> 1.0: 3.0 is 3x the physical ceiling. Foe acts 2
///   of every 4 turns (mover alternates per 2-action turn), so the 4-turn
///   trailing build rate caps at ~1.0; observed max exactly 1.00 in 18/24.
/// - BANK area_sum entry gains BANK_MIN_TURN guard: P0 fired at t=6-19 on
///   opening loops in 24/24 games (sticky, masked all other phases).
/// - BUILD exit bypasses MIN_DWELL (structural): dwell gates leaving active
///   phases, not the first exit from the default; P0 dwell blocked the
///   turns-1-2 rush signature the spec targets.
const W: usize = 4;
const CONTEST_ENTER_RATE: f64 = 0.5;
const CONTEST_EXIT_RATE: f64 = 0.4;
const CONTEST_EXIT_QUIET_TURNS: u32 = 4;
const CONTEST_MAX_TURN: u8 = 40;
const CONTEST_FORCE_EXIT_TURN: u8 = 60;
const WALL_ENTER_BUILD: f64 = 1.0;
const WALL_ENTER_AREA: f64 = 1.0;
const WALL_EXIT_BUILD_HALF: f64 = 0.5;
const BANK_AREA_SUM: f64 = 25.0;
const BANK_MIN_TURN: u8 = 30;
const BANK_FORCE_TURN: u8 = 60;
const BANK_FORCE_LEAD: f64 = 200.0;
const DENY_ENTER_LEAD: f64 = 300.0;
const DENY_EXIT_LEAD: f64 = 150.0;
const MIN_DWELL_TURNS: u8 = 4;
const CONTEST_COOLDOWN_TURNS: u8 = 8;
/// "Big close": GB-anchored minimum bankable loop (opp_gbstyle MIN_CLOSE).
const BIG_CLOSE_AREA: f64 = 8.0;
/// Minimum turns of history before rate triggers may fire (warmup guard).
const MIN_HIST_TURNS: usize = 2;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Phase {
    Build,
    Contest,
    WallRace,
    BankRace,
    Deny,
}

impl Phase {
    fn name(self) -> &'static str {
        match self {
            Phase::Build => "BUILD",
            Phase::Contest => "CONTEST",
            Phase::WallRace => "WALL-RACE",
            Phase::BankRace => "BANK-RACE",
            Phase::Deny => "DENY",
        }
    }
}

#[derive(Clone, Debug)]
struct TurnSnapshot {
    turn: u8,
    score_us: f64,
    score_foe: f64,
    area_us: f64,
    area_foe: f64,
    edges_foe: usize,
    fresh_foe: usize,
    breaks_by_foe: u32,
    closes_by_foe: u32,
}

struct PhaseTracker {
    us: Player,
    foe: Player,
    hist: Vec<TurnSnapshot>,
    phase: Phase,
    dwell: u8,
    quiet_turns: u32, // consecutive turns with foe_close_rate < exit (CONTEST exit)
    contest_cool: u8, // turns remaining on CONTEST re-entry cooldown
    transitions: Vec<(u8, &'static str, &'static str, String)>,
    // Diagnostics (measurement only, never feed triggers).
    max_close_rate: f64,
    max_close_rate_turn: u8,
    max_build_rate: f64,
    max_build_rate_turn: u8,
    max_area_rate: f64,
    max_area_rate_turn: u8,
    first_close_turn: Option<u8>,
    n_small_closes: u32, // foe Connect gain>0 but < BIG_CLOSE_AREA
}

impl PhaseTracker {
    fn new(us: Player) -> Self {
        Self {
            us,
            foe: us.opponent(),
            hist: Vec::new(),
            phase: Phase::Build,
            dwell: 0,
            quiet_turns: 0,
            contest_cool: 0,
            transitions: Vec::new(),
            max_close_rate: 0.0,
            max_close_rate_turn: 0,
            max_build_rate: 0.0,
            max_build_rate_turn: 0,
            max_area_rate: 0.0,
            max_area_rate_turn: 0,
            first_close_turn: None,
            n_small_closes: 0,
        }
    }

    fn last(&self) -> Option<&TurnSnapshot> {
        self.hist.last()
    }

    /// Rate of `f` over the last W turns (or fewer early; None if < warmup).
    fn rate(&self, f: impl Fn(&TurnSnapshot) -> f64) -> Option<f64> {
        let n = self.hist.len();
        if n < MIN_HIST_TURNS + 1 {
            return None;
        }
        let w = W.min(n - 1);
        let now = f(&self.hist[n - 1]);
        let then = f(&self.hist[n - 1 - w]);
        Some((now - then) / w as f64)
    }

    fn foe_close_rate(&self) -> Option<f64> {
        self.rate(|s| s.closes_by_foe as f64)
    }
    fn foe_build_rate(&self) -> Option<f64> {
        self.rate(|s| s.edges_foe as f64)
    }
    fn foe_area_rate(&self) -> Option<f64> {
        self.rate(|s| s.area_foe)
    }
    fn bank_lead(&self) -> Option<f64> {
        self.last().map(|s| s.score_us - s.score_foe)
    }

    fn set_phase(&mut self, turn: u8, next: Phase, why: String) {
        if next == self.phase {
            return;
        }
        self.transitions
            .push((turn, self.phase.name(), next.name(), why));
        if self.phase == Phase::Contest {
            self.contest_cool = CONTEST_COOLDOWN_TURNS;
        }
        self.phase = next;
        self.dwell = 0;
        self.quiet_turns = 0;
    }

    fn push(&mut self, snap: TurnSnapshot) {
        let turn = snap.turn;
        self.hist.push(snap);
        self.dwell = self.dwell.saturating_add(1);
        self.contest_cool = self.contest_cool.saturating_sub(1);

        let close_rate = self.foe_close_rate();
        let build_rate = self.foe_build_rate();
        let area_rate = self.foe_area_rate();
        // Diagnostics: max trailing rates (do not feed triggers).
        if let Some(r) = close_rate {
            if r > self.max_close_rate {
                self.max_close_rate = r;
                self.max_close_rate_turn = turn;
            }
        }
        if let Some(r) = build_rate {
            if r > self.max_build_rate {
                self.max_build_rate = r;
                self.max_build_rate_turn = turn;
            }
        }
        if let Some(r) = area_rate {
            if r > self.max_area_rate {
                self.max_area_rate = r;
                self.max_area_rate_turn = turn;
            }
        }
        let lead = self.bank_lead().unwrap_or(0.0);
        let s = self.last().unwrap();
        let area_sum = s.area_us + s.area_foe;

        // Track CONTEST exit quiet streak regardless of phase.
        match close_rate {
            Some(r) if r < CONTEST_EXIT_RATE => self.quiet_turns += 1,
            Some(_) => self.quiet_turns = 0,
            None => {}
        }

        let can_switch = self.dwell >= MIN_DWELL_TURNS;
        match self.phase {
            Phase::Deny => {
                if lead < DENY_EXIT_LEAD && can_switch {
                    self.set_phase(turn, Phase::BankRace, format!("lead {lead:.0} < 150"));
                }
            }
            Phase::BankRace => {
                if lead >= DENY_ENTER_LEAD && can_switch {
                    self.set_phase(turn, Phase::Deny, format!("lead {lead:.0} >= 300"));
                }
                // BANK-RACE is otherwise sticky to game end (spec).
            }
            Phase::Contest => {
                let rate_exit = self.quiet_turns >= CONTEST_EXIT_QUIET_TURNS;
                let turn_exit = turn >= CONTEST_FORCE_EXIT_TURN;
                if (rate_exit || turn_exit) && can_switch {
                    let why = if turn_exit {
                        format!("turn {turn} >= 60")
                    } else {
                        format!("close_rate<0.4 x{}", self.quiet_turns)
                    };
                    self.phase = Phase::Build; // fall through to re-evaluate below
                    self.transitions.push((turn, "CONTEST", "BUILD", why));
                    self.contest_cool = CONTEST_COOLDOWN_TURNS;
                    self.dwell = 0;
                    self.quiet_turns = 0;
                    self.eval_entry(turn, close_rate, build_rate, area_rate, lead, area_sum);
                }
            }
            Phase::WallRace => {
                let s = self.last().unwrap();
                let n = self.hist.len();
                let closes_this_turn = s.closes_by_foe
                    - if n >= 2 {
                        self.hist[n - 2].closes_by_foe
                    } else {
                        0
                    };
                let banked = closes_this_turn >= 2;
                let halved = build_rate.map_or(false, |r| r < WALL_EXIT_BUILD_HALF);
                if (banked || halved) && can_switch {
                    let why = if banked {
                        format!("foe closes {closes_this_turn} in a turn")
                    } else {
                        format!("build_rate {:.2} < 1.5", build_rate.unwrap_or(0.0))
                    };
                    self.transitions.push((turn, "WALL-RACE", "BUILD", why));
                    self.phase = Phase::Build;
                    self.dwell = 0;
                    self.eval_entry(turn, close_rate, build_rate, area_rate, lead, area_sum);
                }
            }
            Phase::Build => {
                // P1: first exit from the default is immediate (dwell gates
                // active phases only) so turns-1-2 rush signatures are seen.
                self.eval_entry(turn, close_rate, build_rate, area_rate, lead, area_sum);
            }
        }
    }

    /// Entry evaluation from BUILD (priority: DENY > CONTEST > WALL > BANK).
    fn eval_entry(
        &mut self,
        turn: u8,
        close_rate: Option<f64>,
        build_rate: Option<f64>,
        area_rate: Option<f64>,
        lead: f64,
        area_sum: f64,
    ) {
        let bank_ready = (area_sum >= BANK_AREA_SUM && turn >= BANK_MIN_TURN)
            || (turn >= BANK_FORCE_TURN && lead.abs() < BANK_FORCE_LEAD);
        // DENY only via BANK-RACE, but allow direct BUILD->DENY if both hold.
        if bank_ready && lead >= DENY_ENTER_LEAD {
            self.set_phase(turn, Phase::Deny, format!("bank + lead {lead:.0} >= 300"));
            return;
        }
        if let Some(r) = close_rate {
            if r >= CONTEST_ENTER_RATE && turn < CONTEST_MAX_TURN && self.contest_cool == 0 {
                self.set_phase(turn, Phase::Contest, format!("close_rate {r:.2} >= {CONTEST_ENTER_RATE}"));
                return;
            }
        }
        if let (Some(b), Some(a)) = (build_rate, area_rate) {
            if b >= WALL_ENTER_BUILD && a < WALL_ENTER_AREA {
                self.set_phase(
                    turn,
                    Phase::WallRace,
                    format!("build {b:.2} >= {WALL_ENTER_BUILD}, area {a:.2} < {WALL_ENTER_AREA}"),
                );
                return;
            }
        }
        if bank_ready {
            self.set_phase(turn, Phase::BankRace, format!("area_sum {area_sum:.0} / turn {turn}"));
        }
    }
}

/// Foe fresh-wall proxy: shielded edges touching a foe node (no owner on
/// Edge; mirrors search's near_fresh_enemy approximation).
fn fresh_foe(pos: &Position, foe: Player) -> usize {
    pos.shielded_edges()
        .filter(|e| {
            pos.node_owner(e.origin()) == Some(foe) || pos.node_owner(e.far()) == Some(foe)
        })
        .count()
}

struct BigClose {
    turn: u8,
    gain: f64,
}

struct GameResult {
    label: String,
    us_blue: bool,
    skip: usize,
    rs: f64,
    ss: f64,
    win: bool,
    dwells: [(Phase, u32); 5],
    transitions: Vec<(u8, &'static str, &'static str, String)>,
    big_closes: Vec<BigClose>,
    contest_turns: u32,
    wall_turns: u32,
    /// CONTEST-active turns whose trailing close_rate < 0.4 (exit lag).
    contest_lag_turns: u32,
    /// Phase active at each big close: index into big_closes -> phase name.
    close_phases: Vec<&'static str>,
    /// Lead time: turns from most recent CONTEST/WALL entry to each big close
    /// (None if no such entry before the close).
    close_leads: Vec<Option<u8>>,
    max_close_rate: f64,
    max_close_rate_turn: u8,
    max_build_rate: f64,
    max_build_rate_turn: u8,
    max_area_rate: f64,
    max_area_rate_turn: u8,
    first_close_turn: Option<u8>,
    n_small_closes: u32,
}

fn foe_move(label: &str, pos: &Position) -> Option<Move> {
    match label {
        "collapser" => opp_collapser::best_move(pos),
        "gbstyle" => opp_gbstyle::best_move(pos),
        "v1base" => v1base::best_move(pos),
        _ => None,
    }
}

fn play(label: &str, us_blue: bool, skip: usize) -> GameResult {
    let us = if us_blue { Player::Blue } else { Player::Red };
    let foe = us.opponent();
    let mut game = Game::new();
    for _ in 0..skip {
        if game.is_over() {
            break;
        }
        let mv = retaliator::search::best_move(game.position()).unwrap();
        game.play(mv).unwrap();
    }
    let mut tr = PhaseTracker::new(us);
    let mut breaks_by_foe: u32 = 0;
    let mut closes_by_foe: u32 = 0;
    let mut big_closes: Vec<BigClose> = Vec::new();
    // Phase at push-time per turn (turn -> phase after update).
    let mut phase_at_turn: Vec<(u8, Phase, Option<f64>)> = Vec::new();
    // Entry turns for lead-time computation.
    let mut cw_entries: Vec<u8> = Vec::new();

    // Opening snapshot (turn 0 baseline for rates).
    {
        let p = game.position();
        tr.hist.push(TurnSnapshot {
            turn: p.turn_index(),
            score_us: p.score(us).to_f64(),
            score_foe: p.score(foe).to_f64(),
            area_us: p.area(us).to_f64(),
            area_foe: p.area(foe).to_f64(),
            edges_foe: p.edges(foe).len(),
            fresh_foe: fresh_foe(p, foe),
            breaks_by_foe: 0,
            closes_by_foe: 0,
        });
    }

    while !game.is_over() {
        let mover = game.position().to_move();
        let ret_moves = (mover == Player::Blue) == us_blue;
        let area_before = game.position().area(mover).to_f64();
        let mv = if ret_moves {
            retaliator::search::best_move(game.position())
        } else {
            foe_move(label, game.position())
        };
        let outcome = game.play(mv.unwrap()).unwrap();
        if outcome.broken.is_some() && mover == foe {
            breaks_by_foe += 1;
        }
        if outcome.kind == MoveKind::Connect && mover == foe {
            let gain = game.position().area(foe).to_f64() - area_before;
            if gain > 0.0 {
                closes_by_foe += 1;
                if tr.first_close_turn.is_none() {
                    tr.first_close_turn = Some(game.position().turn_index());
                }
                if gain >= BIG_CLOSE_AREA {
                    big_closes.push(BigClose {
                        turn: game.position().turn_index(),
                        gain,
                    });
                } else {
                    tr.n_small_closes += 1;
                }
            }
        }
        if outcome.turn_ended || outcome.game_over {
            let p = game.position();
            let prev_phase = tr.phase;
            tr.push(TurnSnapshot {
                turn: p.turn_index(),
                score_us: p.score(us).to_f64(),
                score_foe: p.score(foe).to_f64(),
                area_us: p.area(us).to_f64(),
                area_foe: p.area(foe).to_f64(),
                edges_foe: p.edges(foe).len(),
                fresh_foe: fresh_foe(p, foe),
                breaks_by_foe,
                closes_by_foe,
            });
            if (tr.phase == Phase::Contest || tr.phase == Phase::WallRace)
                && prev_phase != tr.phase
            {
                cw_entries.push(p.turn_index());
            }
            phase_at_turn.push((p.turn_index(), tr.phase, tr.foe_close_rate()));
        }
    }

    let p = game.position();
    let rs = p.score(us).to_f64();
    let ss = p.score(foe).to_f64();
    let mut dw = [
        (Phase::Build, 0u32),
        (Phase::Contest, 0),
        (Phase::WallRace, 0),
        (Phase::BankRace, 0),
        (Phase::Deny, 0),
    ];
    let mut contest_turns = 0u32;
    let mut wall_turns = 0u32;
    let mut contest_lag = 0u32;
    for &(_, ph, rate) in &phase_at_turn {
        for d in dw.iter_mut() {
            if d.0 == ph {
                d.1 += 1;
            }
        }
        if ph == Phase::Contest {
            contest_turns += 1;
            if rate.map_or(false, |r| r < CONTEST_EXIT_RATE) {
                contest_lag += 1;
            }
        }
        if ph == Phase::WallRace {
            wall_turns += 1;
        }
    }
    let phase_of = |t: u8| -> &'static str {
        phase_at_turn
            .iter()
            .rev()
            .find(|&&(pt, _, _)| pt <= t)
            .map(|&(_, ph, _)| ph.name())
            .unwrap_or("BUILD")
    };
    let mut close_phases = Vec::new();
    let mut close_leads = Vec::new();
    for bc in &big_closes {
        close_phases.push(phase_of(bc.turn));
        close_leads.push(cw_entries.iter().filter(|&&e| e <= bc.turn).max().map(|&e| bc.turn - e));
    }

    GameResult {
        label: label.to_string(),
        us_blue,
        skip,
        rs,
        ss,
        win: rs > ss,
        dwells: dw,
        transitions: std::mem::take(&mut tr.transitions),
        big_closes,
        contest_turns,
        wall_turns,
        contest_lag_turns: contest_lag,
        close_phases,
        close_leads,
        max_close_rate: tr.max_close_rate,
        max_close_rate_turn: tr.max_close_rate_turn,
        max_build_rate: tr.max_build_rate,
        max_build_rate_turn: tr.max_build_rate_turn,
        max_area_rate: tr.max_area_rate,
        max_area_rate_turn: tr.max_area_rate_turn,
        first_close_turn: tr.first_close_turn,
        n_small_closes: tr.n_small_closes,
    }
}

fn main() {
    let mut all: Vec<GameResult> = Vec::new();
    for label in ["collapser", "gbstyle", "v1base"] {
        for &skip in &[0usize, 10, 20, 30] {
            for &us_blue in &[true, false] {
                let r = play(label, us_blue, skip);
                println!(
                    "GAME {} us={} skip={} rs={:.0} ss={:.0} {} dwell B/C/W/K/D={}/{}/{}/{}/{} trans={} bigcloses={} contest_turns={} wall_turns={} lag={} maxclose={:.2}@{} maxbuild={:.2}@{} maxarea={:.2}@{} firstclose={} small={}",
                    r.label,
                    if r.us_blue { "blue" } else { "red" },
                    r.skip,
                    r.rs,
                    r.ss,
                    if r.win { "WIN" } else { "LOSS" },
                    r.dwells[0].1,
                    r.dwells[1].1,
                    r.dwells[2].1,
                    r.dwells[3].1,
                    r.dwells[4].1,
                    r.transitions.len(),
                    r.big_closes.len(),
                    r.contest_turns,
                    r.wall_turns,
                    r.contest_lag_turns,
                    r.max_close_rate,
                    r.max_close_rate_turn,
                    r.max_build_rate,
                    r.max_build_rate_turn,
                    r.max_area_rate,
                    r.max_area_rate_turn,
                    r.first_close_turn.map_or("none".to_string(), |t| t.to_string()),
                    r.n_small_closes,
                );
                for (t, a, b, why) in &r.transitions {
                    println!("  t={t:3} {a:9} -> {b:9} ({why})");
                }
                for (i, bc) in r.big_closes.iter().enumerate() {
                    println!(
                        "  CLOSE t={:3} gain={:5.1} phase={:9} lead={}",
                        bc.turn,
                        bc.gain,
                        r.close_phases[i],
                        r.close_leads[i]
                            .map_or("none".to_string(), |l| format!("{l} turns")),
                    );
                }
                all.push(r);
            }
        }
    }
    // Aggregate per line.
    for label in ["collapser", "gbstyle", "v1base"] {
        let gs: Vec<&GameResult> = all.iter().filter(|g| g.label == label).collect();
        let w = gs.iter().filter(|g| g.win).count();
        let md = |i: usize| {
            let mut v: Vec<u32> = gs.iter().map(|g| g.dwells[i].1).collect();
            v.sort_unstable();
            let sum: u32 = v.iter().sum();
            (v[0], v[v.len() / 2], v[v.len() - 1], sum as f64 / v.len() as f64)
        };
        let (b0, b1, b2, b3) = md(0);
        let (c0, c1, c2, c3) = md(1);
        let (w0, w1, w2, w3) = md(2);
        let (k0, k1, k2, k3) = md(3);
        let (d0, d1, d2, d3) = md(4);
        let nclose: usize = gs.iter().map(|g| g.big_closes.len()).sum();
        let with_lead = gs
            .iter()
            .flat_map(|g| g.close_leads.iter())
            .filter(|l| l.is_some())
            .count();
        let leads: Vec<u8> = gs
            .iter()
            .flat_map(|g| g.close_leads.iter().filter_map(|&l| l))
            .collect();
        let mean_lead = if leads.is_empty() {
            -1.0
        } else {
            leads.iter().map(|&l| l as f64).sum::<f64>() / leads.len() as f64
        };
        let clag: u32 = gs.iter().map(|g| g.contest_lag_turns).sum();
        let ctot: u32 = gs.iter().map(|g| g.contest_turns).sum();
        println!(
            "AGG {} n={} W-L={}-{} bigcloses={} w/lead={}/{} mean_lead={:.1} | dwell BUILD min/med/max/mean={}/{}/{}/{:.1} CONTEST={}/{}/{}/{:.1} WALL={}/{}/{}/{:.1} BANK={}/{}/{}/{:.1} DENY={}/{}/{}/{:.1} | contest_lag={}/{}",
            label, gs.len(), w, gs.len() - w, nclose, with_lead, nclose, mean_lead,
            b0, b1, b2, b3, c0, c1, c2, c3, w0, w1, w2, w3, k0, k1, k2, k3, d0, d1, d2, d3,
            clag, ctot,
        );
        // Color split.
        for blue in [true, false] {
            let cs: Vec<&GameResult> = gs.iter().filter(|g| g.us_blue == blue).copied().collect();
            let cw = cs.iter().filter(|g| g.win).count();
            let cc: u32 = cs.iter().map(|g| g.contest_turns).sum();
            let ww: u32 = cs.iter().map(|g| g.wall_turns).sum();
            let nc: usize = cs.iter().map(|g| g.big_closes.len()).sum();
            println!(
                "  color us={}: W-L={}-{} contest_turns={} wall_turns={} bigcloses={}",
                if blue { "blue" } else { "red" },
                cw,
                cs.len() - cw,
                cc,
                ww,
                nc,
            );
        }
    }
}
