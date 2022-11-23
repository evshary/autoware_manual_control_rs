fn print_help() {
  println!("------------------------------------");
  println!("| Different Mode:                  |");
  println!("|   z: Toggle auto & external mode |");
  println!("|   x: GateMode => Drive           |");
  println!("|   c: GateMode => Reverse         |");
  println!("|   v: GateMode => Park            |");
  println!("|   s: View current mode           |");
  println!("| Speed:                           |");
  println!("|   u: Increase speed              |");
  println!("|   i: Set speed to 0              |");
  println!("|   o: Decrease speed              |");
  println!("| Steering Angle                   |");
  println!("|   j: Left turn                   |");
  println!("|   k: Set angle to 0              |");
  println!("|   l: Right turn                  |");
  println!("------------------------------------");
}

fn main() {
    print_help();
}
