use std::fs::File;

use clap::{App, Arg};

use crate::intermediary::Intermediary;
use crate::mojangmap::MojangMap;

mod intermediary;
mod jvmsig;
mod mapping_output;
mod mojangmap;

fn main() {
  let matches = App::new("mojang2tiny")
    .arg(Arg::with_name("intermediary")
      .short("i")
      .long("intermediary")
      .help("a file containing notch -> intermediary mappings")
      .value_name("FILE")
      .required(true))
    .arg(Arg::with_name("mappings")
      .short("m")
      .long("mappings")
      .help("a file containing source -> ProGuard mappings")
      .value_name("FILE")
      .required(true)
      .multiple(true).number_of_values(1))
    .arg(Arg::with_name("output-dir")
      .short("o")
      .long("output-dir")
      .help("output directory")
      .value_name("DIRECTORY")
      .required(true))
    .get_matches();

  let intermediary = matches.value_of("intermediary").unwrap();

  println!("Loading intermediary");
  let file = File::open(intermediary).unwrap();
  let i = Intermediary::load(file);

  let mojang = matches.values_of("mappings").unwrap();

  let m = mojang.fold(MojangMap::empty(), |mut acc, it| {
    println!("Loading mojang");
    let file = File::open(it).unwrap();
    let m = MojangMap::load(file);
    acc.combine(m);
    acc
  });

  let out_dir = matches.value_of("output-dir").unwrap();

  println!("Writing mapping files");
  mapping_output::write(out_dir, &i, &m);
}
