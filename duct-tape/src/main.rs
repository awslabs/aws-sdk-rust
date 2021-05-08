use duct::cmd;

fn main() {
   let stdout = cmd!("aws", "--version").read();
   assert_eq!(stdout.unwrap(), "llama");
}
