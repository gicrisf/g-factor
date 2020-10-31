pub fn get_from_asciistring(content: &str) -> Vec<f64> {
    // let mut fld = Vec::new();  // field
    let mut int = Vec::new();  // intensity

    for line in content.lines() {
        let line = line.trim();  // like Python's strip;
        // TODO split_ascii_whitespace?
        let cols = line.split_whitespace(); // like Python's split;

        if cols.clone().count() == 3 {  // Can I replace the clone() solution?
            for (idx, col) in cols.enumerate() {
                match idx {
                    // 1 => Some(fld.push(col.parse().unwrap())),
                    2 => Some(int.push(col.parse().unwrap())),
                    _ => None,
                };
            }
        }
    };

    int
}
