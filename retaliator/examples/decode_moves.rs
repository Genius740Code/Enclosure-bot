use meridian_engine::{Move, Point};

fn main() {
    // GB Blue moves from 30c7653b
    let moves = vec![
        896, 2713, 12419, 13892, 16780, 17147, 14946, 16819, 16839, 2767,
        4933, 16878, 16898, 2406, 205, 11522, 1979, 15712, 14284, 4145,
    ];
    println!("GB Blue (30c7653b) first 20 moves:");
    for id in moves {
        if let Some(mv) = Move::from_index(id as usize) {
            let src = mv.source;
            let tgt = mv.target().unwrap();
            println!("  {}: {:?}->{:?}", id, src, tgt);
        }
    }
    
    // GB Red moves from ad65f054
    let moves2 = vec![
        896, 13182, 15348, 16058, 4867, 15385, 15403, 14579, 14981, 15440,
        15458, 17106, 4927, 10802, 547, 234, 14639, 3435, 508, 4864,
    ];
    println!("\nGB Red (ad65f054) first 20 moves:");
    for id in moves2 {
        if let Some(mv) = Move::from_index(id as usize) {
            let src = mv.source;
            let tgt = mv.target().unwrap();
            println!("  {}: {:?}->{:?}", id, src, tgt);
        }
    }
}
