use meridian_engine::{Game, Move};
fn main() {
    let path = std::env::args().nth(1).unwrap();
    let upto: usize = std::env::args().nth(2).unwrap().parse().unwrap();
    let d: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let ids: Vec<usize> = d["moves"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as usize).collect();
    let mut game = Game::new();
    for &id in &ids[..upto] {
        game.play(Move::from_index(id).unwrap()).unwrap();
    }
    let pos = game.position().clone();
    let plain = retaliator::search::analyze(&pos, 4096);
    let p0: Vec<usize> = plain.candidates.iter().map(|c| c.mv.index()).collect();
    let tgt = retaliator::search::best_move(&pos).unwrap().target().unwrap();
    let dodged = retaliator::search::analyze_with_avoid(&pos, 4096, &[tgt]);
    let p1: Vec<usize> = dodged.candidates.iter().map(|c| c.mv.index()).collect();
    println!("no-avoid top8:  {p0:?}");
    println!("with-avoid top8: {p1:?}");
}
