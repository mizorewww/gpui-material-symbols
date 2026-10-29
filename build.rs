use std::{collections::BTreeMap, env, fs, path::PathBuf};

fn main() {
    let mut catalog = BTreeMap::<String, [u32; 3]>::new();
    for (index, style) in ["Outlined", "Rounded", "Sharp"].iter().enumerate() {
        let path = format!("assets/MaterialSymbols{style}.codepoints");
        println!("cargo:rerun-if-changed={path}");
        for line in fs::read_to_string(path).unwrap().lines() {
            let mut fields = line.split_whitespace();
            let name = fields.next().unwrap();
            let codepoint = u32::from_str_radix(fields.next().unwrap(), 16).unwrap();
            assert!(char::from_u32(codepoint).is_some());
            let entry = catalog.entry(name.to_owned()).or_default();
            assert_eq!(entry[index], 0, "duplicate symbol {name}");
            entry[index] = codepoint;
        }
    }
    println!("cargo:rerun-if-changed=assets/supplemental.tsv");
    let supplemental = fs::read_to_string("assets/supplemental.tsv").unwrap();
    for line in supplemental.lines() {
        let name = line.split('/').next().unwrap();
        catalog.entry(name.to_owned()).or_default();
    }
    let mut output = String::from("impl Symbol {\n");
    for (index, (name, codepoints)) in catalog.iter().enumerate() {
        assert!(codepoints.iter().all(|cp| *cp != 0) || codepoints.iter().all(|cp| *cp == 0));
        let mut ident = name.to_ascii_uppercase();
        if ident.starts_with(|c: char| c.is_ascii_digit()) {
            ident.insert(0, '_');
        }
        output.push_str(&format!(
            "/// The `{name}` symbol.\npub const {ident}: Self = Self({index});\n"
        ));
    }
    output.push_str("}\npub(crate) static CATALOG: &[(&str, [u32; 3])] = &[\n");
    for (name, codepoints) in &catalog {
        output.push_str(&format!("({name:?}, {codepoints:?}),\n"));
    }
    output.push_str("];\n");
    output.push_str("pub(crate) static SUPPLEMENTAL: &[(&str, usize, usize)] = &[\n");
    for line in supplemental.lines() {
        let fields: Vec<_> = line.split('\t').collect();
        output.push_str(&format!(
            "({:?}, {}, {}),\n",
            fields[0], fields[1], fields[2]
        ));
    }
    output.push_str("];\n");
    fs::write(
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("catalog.rs"),
        output,
    )
    .unwrap();
}
