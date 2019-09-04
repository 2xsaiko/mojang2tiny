use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::intermediary::{Intermediary, IntermediaryEntry};
use crate::jvmsig::{JvmSignature, JvmType};
use crate::mojangmap::{MojangMap, MojangMapEntry};

pub fn write(dir: impl AsRef<Path>, intermediary: &Intermediary, mojang: &MojangMap) {
  let _ = fs::remove_dir_all(&dir);
  fs::create_dir_all(&dir).unwrap();

  for entry in intermediary.entries.iter() {
    if let IntermediaryEntry::Class { obf_name, int_name } = entry {
      println!("{} -> {}", obf_name, int_name);
    }
  }

  for entry in mojang.entries.iter() {
    if let MojangMapEntry::Class { deobf_name, obf_name } = entry {
      let mut path = dir.as_ref().to_path_buf();
      path.push(deobf_name);
      path.pop();
      fs::create_dir_all(&path).unwrap();
      path.push(&format!("{}.mapping", &deobf_name[deobf_name.rfind('/').map(|it| it + 1).unwrap_or(0)..]));

      let int_name = intermediary.entries.iter().find_map(|it| match it {
        IntermediaryEntry::Class { obf_name: sel, int_name } if sel == obf_name => Some(int_name),
        _ => None,
      });

      if let Some(int_name) = int_name {
        println!(" - Class {}", deobf_name);
        let mut file = File::create(&path).unwrap();

        writeln!(file, "CLASS {} {}", int_name, deobf_name).unwrap();

        let subentries = mojang.entries.iter().filter(|it| match it {
          MojangMapEntry::Method { deobf_class, .. } => deobf_class == deobf_name,
          MojangMapEntry::Field { deobf_class, .. } => deobf_class == deobf_name,
          _ => false,
        });

        for e in subentries {
          match e {
            MojangMapEntry::Field { deobf_class, obf_class, deobf_type, deobf_name, obf_name } => {
              let int_name = intermediary.entries.iter().find_map(|it| match it {
                IntermediaryEntry::Field { obf_class: sel1, obf_name: sel2, int_name, .. } if sel1 == obf_class && sel2 == obf_name => Some(int_name),
                _ => None,
              });

              if let Some(int_name) = int_name {
                println!("   - Field {}", deobf_name);
                let int_type = obf_to_int_type(&deobf_to_obf_type(deobf_type, mojang), intermediary);
                writeln!(file, "\tFIELD {} {} {}", int_name, deobf_name, int_type).unwrap();
              } else {
                eprintln!("Can't find intermediary name for field '{} {}'", deobf_class, deobf_name);
              }
            }
            MojangMapEntry::Method { deobf_class, obf_class, deobf_sig, deobf_name, obf_name } => {
              let int_name = intermediary.entries.iter().find_map(|it| match it {
                IntermediaryEntry::Field { obf_class: sel1, obf_name: sel2, int_name, .. } if sel1 == obf_class && sel2 == obf_name => Some(int_name),
                _ => None,
              });

              if let Some(int_name) = int_name {
                println!("   - Field {}", deobf_name);
                let int_type = obf_to_int_sig(&deobf_to_obf_sig(deobf_sig, mojang), intermediary);
                writeln!(file, "\tMETHOD {} {} {}", int_name, deobf_name, int_type).unwrap();
              } else {
                eprintln!("Can't find intermediary name for method '{} {}{}'", deobf_class, deobf_name, deobf_sig);
              }
            }
            _ => {}
          }
        }
      } else {
        eprintln!("Can't find intermediary name for class '{}'!", obf_name);
      }
    }
  }
}

fn deobf_to_obf_type(t: &JvmType, mojang: &MojangMap) -> JvmType {
  match t {
    JvmType::Class(deobf_name) => {
      let obf_name = mojang.entries.iter().find_map(|it| match it {
        MojangMapEntry::Class { deobf_name: sel, obf_name } if sel == deobf_name => Some(obf_name),
        _ => None,
      });

      JvmType::Class(obf_name.unwrap_or(deobf_name).clone())
    }
    JvmType::Array(t) => JvmType::Array(Box::new(deobf_to_obf_type(t, mojang))),
    _ => t.clone(),
  }
}

fn obf_to_int_type(t: &JvmType, intermediary: &Intermediary) -> JvmType {
  match t {
    JvmType::Class(obf_name) => {
      let int_name = intermediary.entries.iter().find_map(|it| match it {
        IntermediaryEntry::Class { obf_name: sel, int_name } if sel == obf_name => Some(int_name),
        _ => None,
      });

      JvmType::Class(int_name.unwrap_or(obf_name).clone())
    }
    JvmType::Array(t) => JvmType::Array(Box::new(obf_to_int_type(t, intermediary))),
    _ => t.clone(),
  }
}

fn deobf_to_obf_sig(sig: &JvmSignature, mojang: &MojangMap) -> JvmSignature {
  let result = deobf_to_obf_type(sig.result(), mojang);
  let params: Vec<_> = sig.params().iter().map(|it| deobf_to_obf_type(it, mojang)).collect();
  JvmSignature::from(&params, &result)
}

fn obf_to_int_sig(sig: &JvmSignature, intermediary: &Intermediary) -> JvmSignature {
  let result = obf_to_int_type(sig.result(), intermediary);
  let params: Vec<_> = sig.params().iter().map(|it| obf_to_int_type(it, intermediary)).collect();
  JvmSignature::from(&params, &result)
}