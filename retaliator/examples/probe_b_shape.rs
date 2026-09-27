//! Lane B shape probe: what does the shipped search actually DO in the opening,
//! and how does that differ from Great Barrier's wall book (12 wall actions,
//! first close act 13-15, ~10 area/loop)? Grounds the mobility / potential-area
//! term design ("long sticks" vs tiny closes) in numbers before any term is
//! added. Read-only over the engine: no eval terms here.
//!
//! Lines replayed (both deterministic):
//!   1. search self-play from the empty start (the scoreboard league skip=0 game)
//!   2. GB book as Blue (act 1 = D10-F11, then the 12 book moves) vs search as Red
//! For every action of line 2 the probe also prints what the shipped search
//! would pick at that position — showing where search refuses the wall.
//! Usage: cargo run --release --example probe_b_shape

use meridian_engine::{Game, Move, MoveKind, Player, Point, Position};

/// GB's mirrored wall book, Blue side (rival-analysis §4): act 2-13.
/// Act 1 varies per game (D10-F11 here); the 12 book moves follow.
const GB_BOOK: &[(&str, i8, i8)] = &[
    ("D10-E12", -5, 2),
    ("D10-F13", -4, 3),
    ("E12-G15", -3, 5),
    ("F13-H16", -2, 6),
    ("G15-I18", -1, 8),
    ("H16-J19", 0, 9),
    ("I18-J19", 0, 9),
    ("D10-F7", -4, -3),
    ("D10-E8", -5, -2),
    ("E8-G5", -3, -5),
    ("F7-H4", -2, -6),
    ("G5-I2", -1, -8),
];

fn hull_area(position: &Position, player: Player) -> f64 {
    let mut points: Vec<Point> = position.nodes(player).iter().collect();
    points.sort_by_key(|point| (point.x(), point.y()));
    if points.len() < 3 {
        return 0.0;
    }
    let mut hull = half_hull(points.iter().copied());
    hull.extend(half_hull(points.iter().rev().copied()));
    let first = hull[0];
    hull.windows(2).map(|pair| cross(first, pair[0], pair[1]) as f64).sum::<f64>().abs() * 0.5
}

fn half_hull(points: impl Iterator<Item = Point>) -> Vec<Point> {
    let mut chain: Vec<Point> = Vec::new();
    for point in points {
        while chain.len() >= 2 && cross(chain[chain.len() - 2], chain[chain.len() - 1], point) <= 0 {
            chain.pop();
        }
        chain.push(point);
    }
    chain.pop();
    chain
}

fn cross(a: Point, b: Point, c: Point) -> i32 {
    i32::from(b.x() - a.x()) * i32::from(c.y() - a.y())
        - i32::from(b.y() - a.y()) * i32::from(c.x() - a.x())
}

fn length(mv: Move) -> i8 {
    mv.direction.dx().abs().max(mv.direction.dy().abs())
}

/// Per-action shape stats for `mover`, appended to the log lines.
struct Shape {
    first_close: Option<u32>,
    closes: u32,
    cuts_taken: u32,
}

fn replay_line(
    label: &str,
    mut game: Game,
    blue_book: bool,
    watch: Player,
    probe_search_picks: bool,
) -> Shape {
    let mut shape = Shape { first_close: None, closes: 0, cuts_taken: 0 };
    let mut book_iter = GB_BOOK.iter();
    while !game.is_over() {
        let to_move = game.position().to_move();
        let action = game.position().actions_played();
        let mv = if blue_book && to_move == Player::Blue {
            // Act 1 (Blue solo): D10-F11; then the book, one move per action.
            if action == 0 {
                Move::between(Point::new(-4, 1).expect("F11 on board"), Point::new(-6, 0).expect("D10 on board"))
                    .and_then(|mv| game.position().check_move(mv).ok().map(|_| mv))
            } else {
                book_iter
                    .next()
                    .and_then(|(_, x, y)| {
                        let target = Point::new(*x, *y).expect("book point on board");
                        // The book lists each edge from its earlier endpoint; the
                        // engine move is between the node we hold and the target.
                        game.position()
                            .nodes(Player::Blue)
                            .iter()
                            .filter_map(|src| Move::between(src, target))
                            .find(|mv| game.position().check_move(*mv).is_ok())
                    })
            }
        } else if blue_book {
            retaliator::search::best_move(game.position())
        } else {
            retaliator::search::best_move(game.position())
        }
        .unwrap_or_else(|| game.position().legal_moves().iter().next().expect("a legal move"));
        if blue_book && to_move == Player::Blue && probe_search_picks {
            let picked = retaliator::search::best_move(game.position());
            let agree = picked.map(|p| p.index()) == Some(mv.index());
            println!(
                "{label} act={action} book={} search_picks={} {}",
                mv.index(),
                picked.map(|p| p.index()).map(|i| i.to_string()).unwrap_or_else(|| "-".into()),
                if agree { "AGREE" } else { "DIVERGE" },
            );
        }
        let before_area = game.position().area(to_move).to_f64();
        let before_hull = hull_area(game.position(), to_move);
        let outcome = game.play(mv).expect("move is legal");
        let kind = outcome.kind;
        if to_move == watch {
            let after_area = game.position().area(watch).to_f64();
            let after_hull = hull_area(game.position(), watch);
            if kind == MoveKind::Connect {
                shape.closes += 1;
                if shape.first_close.is_none() {
                    shape.first_close = Some(u32::from(action));
                }
            }
            if outcome.broken.is_some() {
                shape.cuts_taken += 1;
            }
            println!(
                "{label} act={action} {} {} len={} gain={:+.1} hull {bh:.1}->{ah:.1} (d{dh:+.1}) mob={mob} area_now={area:.1}",
                mv.index(),
                match kind { MoveKind::Connect => "close", MoveKind::Extend => "extend", MoveKind::Capture => "capture" },
                length(mv),
                after_area - before_area,
                dh = after_hull - before_hull,
                mob = game.position().legal_moves().iter().count(),
                bh = before_hull,
                ah = after_hull,
                area = after_area,
            );
        }
    }
    let b = game.position().score(Player::Blue).to_f64();
    let r = game.position().score(Player::Red).to_f64();
    let area_b = game.position().area(Player::Blue).to_f64();
    let area_r = game.position().area(Player::Red).to_f64();
    println!("{label} FINAL blue {b:.0}-{r:.0} area {area_b:.1}/{area_r:.1}");
    shape
}

fn main() {
    println!("=== line 1: search self-play, empty start (watch=Blue) ===");
    let s1 = replay_line("self", Game::new(), false, Player::Blue, false);
    println!(
        "self-play Blue: first_close=act {:?} closes={} cuts_taken={}",
        s1.first_close, s1.closes, s1.cuts_taken
    );

    println!("=== line 2: GB book as Blue vs search as Red (watch=Blue) ===");
    let s2 = replay_line("gb", Game::new(), true, Player::Blue, true);
    println!(
        "GB-book Blue: first_close=act {:?} closes={} cuts_taken={}",
        s2.first_close, s2.closes, s2.cuts_taken
    );
    println!(
        "(GB site reference: first close act 13-15, 10-13 closes, 108-121 area, ~10/loop)"
    );
}
