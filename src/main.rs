use std::io;

fn main() {
    // Stage 1: moving from left to right with player.
    // after every input, print new position.
    println!("todge.");

    let mut position: usize = 5;
    let mut state = String::from("...........");
    // state[position] = "@";
    state.replace_range(position..position+1, "@");
    loop { 
        println!("{}", state);
        println!("Move [l/r] or quit [q]:");

        let mut decision_in = String::new();
        io::stdin()
            .read_line(&mut decision_in)
            .expect("Failed...");

        let decision: &str = decision_in.trim();

        // state[position] = ".";
        state.replace_range(position..position+1, ".");

        println!("Got: {decision} with len: {}", decision.len());

        if decision == "l" {
            if position == 0 {
                position = state.len() - 1;
            } else {
                position -= 1;
            }
        } else if decision == "r" {
            position += 1; 
            position %= state.len();
        } else if decision == "q" {
            break;
        }

        // state[position] = "@";
        state.replace_range(position..position+1, "@");
    }
}
