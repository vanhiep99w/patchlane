use std::collections::VecDeque;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io;
use std::path::Path;

pub fn tail_log(path: &Path, max_lines: usize) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut tail = VecDeque::with_capacity(max_lines);

    for line in reader.lines() {
        let line = line?;
        if max_lines == 0 {
            break;
        }
        if tail.len() == max_lines {
            tail.pop_front();
        }
        tail.push_back(line);
    }

    Ok(tail.into_iter().collect())
}
