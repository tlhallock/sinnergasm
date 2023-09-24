// extern crate prost_build;

// fn main() {
//   prost_build::compile_protos(
//     &["src/sinnergasm.proto"],
//     &["src/"]
//   ).unwrap();
// }

fn main() -> Result<(), Box<dyn std::error::Error>> {
  tonic_build::compile_protos("src/sinnergasm.proto")?;
  Ok(())
}
