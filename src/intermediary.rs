use std::io::{BufRead, BufReader, Read};

use crate::jvmsig::{JvmSignature, JvmType};

#[derive(Debug)]
pub struct Intermediary {
  pub entries: Vec<IntermediaryEntry>,
}

#[derive(Debug)]
pub enum IntermediaryEntry {
  Class { obf_name: String, int_name: String },
  Field { obf_class: String, obf_type: JvmType, obf_name: String, int_name: String },
  Method { obf_class: String, obf_sig: JvmSignature, obf_name: String, int_name: String },
}

impl Intermediary {
  pub fn load(reader: impl Read) -> Intermediary {
    let mut reader = BufReader::new(reader);
    let mut buf = String::new();
    let mut entries = Vec::new();
    loop {
      buf.clear();
      let size = reader.read_line(&mut buf).unwrap();
      buf.pop();
      if size == 0 { break; }
      if buf.starts_with('#') { continue; }
      let split: Vec<_> = buf.split('\t').collect();
      if split[0] == "v1" { continue; }
      match split[0] {
        "CLASS" => {
          entries.push(IntermediaryEntry::Class {
            obf_name: split[1].to_owned(),
            int_name: split[2].to_owned(),
          });
        }
        "FIELD" => {
          let class = split[1];
          let mut string = split[2].to_owned();
          let signature = JvmType::read(&mut string);
          let obf_name = split[3];
          let int_name = split[4];
          entries.push(IntermediaryEntry::Field {
            obf_class: class.to_owned(),
            obf_type: signature,
            obf_name: obf_name.to_owned(),
            int_name: int_name.to_owned(),
          });
        }
        "METHOD" => {
          let class = split[1];
          let signature = JvmSignature::from_jvm_sig(split[2]);
          let obf_name = split[3];
          let int_name = split[4];
          entries.push(IntermediaryEntry::Method {
            obf_class: class.to_owned(),
            obf_sig: signature,
            obf_name: obf_name.to_owned(),
            int_name: int_name.to_owned(),
          });
        }
        _ => panic!("Syntax error at: {}", buf)
      }
    }
    Intermediary { entries }
  }
}