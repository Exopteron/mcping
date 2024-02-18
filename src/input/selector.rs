use std::fmt::Display;

#[derive(Debug)]
pub enum Selected<T: Display> {
    Value(T),
    Special(isize)
}



/// Proposes choices to the user for them to select.
pub fn select_one_of<T: Display>(values: impl Iterator<Item = T>) -> std::io::Result<Selected<T>> {
    let mut values_vec = vec![];

    for (index, value) in values.enumerate() {
        println!("* #{index} {value}");
        values_vec.push(value);
    }

    let mut s = String::new();
    let number = loop {
        std::io::stdin().read_line(&mut s)?;
        
        match s.trim().parse::<isize>() {
            // the special case
            Ok(v) if v < 0 => return Ok(Selected::Special(v)),
            // an in-bounds selection
            Ok(v) if (v as usize) < values_vec.len() => break (v as usize),
            // any other (invalid) selection
            _ => {
                println!("Invalid selection: {:?}", s);
                s.clear();
                continue;
            }
        }
    };

    Ok(Selected::Value(values_vec.remove(number)))
}
