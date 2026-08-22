use std::io;


struct Obstacle {
    x: usize,
    y: usize,
}

fn main() {
    // Stage 1: moving from left to right with player.
    // after every input, print new position.
    println!("todge.");

    let mut px: usize = 5;
    const WIDTH: usize = 11;
    const HEIGHT: usize = 11;
    let mut state = [['.'; WIDTH]; HEIGHT];

    state[HEIGHT-1][px] = '@';
    loop { 
        for line in &state {
            for s in line {
                print!("{s}");
            }
            println!();
        }
        println!("Move [l/r] or quit [q]:");

        let mut decision_in = String::new();
        io::stdin()
            .read_line(&mut decision_in)
            .expect("Failed...");

        let decision: &str = decision_in.trim();

        state[HEIGHT-1][px] = '.';

        println!("Got: {decision} with len: {}", decision.len());

        if decision == "l" {
            px = if px != 0 {
                px - 1
            } else { 
                state.len() - 1
            };
        } else if decision == "r" {
            px = (px + 1) % state.len(); 
        } else if decision == "q" {
            break;
        }

        state[HEIGHT-1][px] = '@';
    }
}
