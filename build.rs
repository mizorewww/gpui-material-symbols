use std::{collections::BTreeMap, env, fs, path::PathBuf};

fn main() {
    let mut catalog = BTreeMap::<String, [u32; 3]>::new();
    for (index, style) in ["Outlined", "Rounded", "Sharp"].iter().enumerate() {
        let path = format!("assets/MaterialSymbols{style}.codepoints");
        println!("cargo:rerun-if-changed={path}");
        let codepoints = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"));
        for line in codepoints.lines() {
            let mut fields = line.split_whitespace();
            let (Some(name), Some(codepoint)) = (fields.next(), fields.next()) else {
                panic!("{path}: malformed line {line:?}");
            };
            let codepoint = u32::from_str_radix(codepoint, 16)
                .unwrap_or_else(|_| panic!("{path}: bad codepoint in {line:?}"));
            assert!(
                char::from_u32(codepoint).is_some(),
                "{path}: invalid scalar in {line:?}"
            );
            let entry = catalog.entry(name.to_owned()).or_default();
            assert_eq!(entry[index], 0, "duplicate symbol {name}");
            entry[index] = codepoint;
        }
    }
    println!("cargo:rerun-if-changed=assets/supplemental.tsv");
    let supplemental =
        fs::read_to_string("assets/supplemental.tsv").expect("read assets/supplemental.tsv");
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
    println!("cargo:rerun-if-changed=assets/supplemental.bin");
    let pack_len = fs::metadata("assets/supplemental.bin")
        .expect("read assets/supplemental.bin")
        .len() as usize;
    output.push_str("pub(crate) static SUPPLEMENTAL: &[(&str, usize, usize)] = &[\n");
    let mut previous = "";
    for (number, line) in supplemental.lines().enumerate() {
        let context = || format!("assets/supplemental.tsv:{}", number + 1);
        let [path, offset, len] = line.split('\t').collect::<Vec<_>>()[..] else {
            panic!("{}: expected path, offset and length", context());
        };
        let offset: usize = offset
            .parse()
            .unwrap_or_else(|_| panic!("{}: offset", context()));
        let len: usize = len
            .parse()
            .unwrap_or_else(|_| panic!("{}: length", context()));
        // Icon::supplemental_entry binary-searches this table and slices the pack.
        assert!(
            previous < path,
            "{}: paths must be strictly sorted",
            context()
        );
        assert!(
            offset + len <= pack_len,
            "{}: slice exceeds supplemental.bin",
            context()
        );
        previous = path;
        output.push_str(&format!("({path:?}, {offset}, {len}),\n"));
    }
    output.push_str("];\n");
    fs::write(
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("catalog.rs"),
        output,
    )
    .unwrap();
}
