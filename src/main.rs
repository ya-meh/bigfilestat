mod file_stat {
    use std::cmp::{Ordering};
    use std::collections::VecDeque;
    use std::fs::File;
    use std::io::{prelude::*, BufReader};

    pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    fn for_file<F: FnMut(f64)>(filename: &str, mut action: F) -> Result<()> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);

        for data in reader.split(' ' as u8) {
            action(String::from_utf8(data?)?.parse::<f64>()?);
        }

        Ok(())
    }

    pub fn min_max(filename: &str) -> Result<Option<(f64, f64)>> {
        let mut val_min = None;
        let mut val_max = None;

        for_file(filename, |x| {
            let _ = val_min.insert(x.min(val_min.unwrap_or(x)));
            let _ = val_max.insert(x.max(val_max.unwrap_or(x)));
        })?;

        Ok(val_min.zip(val_max))
    }

    pub fn len(filename: &str) -> Result<usize> {
        let mut size = 0;
        for_file(filename, |_| size += 1)?;
        Ok(size)
    }

    pub fn average(filename: &str) -> Result<f64> {
        let mut sum = 0.0;
        let mut len = 0u64;
        for_file(filename, |x| {
            sum += x;
            len += 1;
        })?;
        Ok(sum / len as f64)
    }

    pub fn dispersion(filename: &str) -> Result<f64> {
        let x_avr = average(filename)?;
        let mut sum = 0.0;
        let mut len = 0u64;
        for_file(filename, |x| {
            sum += (x - x_avr).powi(2);
            len += 1;
        })?;
        Ok(sum / len as f64)
    }


    fn is_median(filename: &str, val: f64) -> std::result::Result<Ordering, Box<dyn std::error::Error>> {
        let mut less = 0i64;
        let mut eq = 0i64;
        let mut greater = 0i64;

        for_file(filename, |x| {
            match x.total_cmp(&val) {
                Ordering::Less => { less += 1 }
                Ordering::Equal => { eq += 1 }
                Ordering::Greater => { greater += 1 }
            }
        })?;

        if (greater - less).abs() < eq + 1 {
            Ok(Ordering::Equal)
        } else if greater < less {
            Ok(Ordering::Less)
        } else {
            Ok(Ordering::Greater)
        }
    }

    fn find_median(filename: &str, left: f64, right: f64) -> Result<f64> {
        if (left - right).abs() <= 0.001 {
            return Ok(left);
        }

        let mid = (right + left) / 2.0;
        match is_median(filename, mid)? {
            Ordering::Less => { find_median(filename, left, mid) }
            Ordering::Equal => { find_median(filename, left, mid) } // ??
            Ordering::Greater => { find_median(filename, mid, right) }
        }
    }

    pub fn median(filename: &str) -> Result<f64> {
        let (min, max) = min_max(filename)?.expect("median requires 1+ value");
        find_median(filename, min, max)
    }

    pub fn tails(filename: &str, len: usize) -> Result<(Vec<f64>, VecDeque<f64>)> {
        let mut left = Vec::new();
        let mut right = VecDeque::new();

        for_file(filename, |x| {
            if left.len() < len {
                left.push(x);
            }

            right.push_back(x);
            if right.len() > len {
                right.pop_front();
            }
        })?;

        Ok((left, right))
    }
}


fn elapsed() -> std::time::Duration {
    unsafe {
        static mut X: Option<std::time::Instant> = None;
        if X.is_none() {
            let _ = X.insert(std::time::Instant::now());
        }
        let ret = X.unwrap().elapsed();
        let _ = X.insert(std::time::Instant::now());
        ret
    }
}


fn main() -> file_stat::Result<()> {
    let start_time = std::time::Instant::now();
    let filename = "testdata/bigfile.txt";
    elapsed();

    println!("LEN\t\t{}\t({:?})", file_stat::len(filename)?, elapsed());
    println!("MIN, MAX\t{:?}\t({:?})", file_stat::min_max(filename)?.expect("no values"), elapsed());
    println!("AVERAGE\t\t{}\t({:?})", file_stat::average(filename)?, elapsed());
    println!("DISPERSION\t{}\t({:?})", file_stat::dispersion(filename)?, elapsed());
    println!("MEDIAN\t\t{}\t({:?})", file_stat::median(filename)?, elapsed());
    let (left, right) = file_stat::tails(filename, 10000)?;
    println!("LEFT TAIL\t{:.3?}\t({:?})", left.iter().take(10).collect::<Vec<&f64>>(), elapsed());
    println!("RIGHT TAIL\t{:.3?}", right.iter().rev().take(10).collect::<Vec<&f64>>());

    println!("TIME TOOK\t{:?}", start_time.elapsed());

    Ok(())
}


/* CHISQ(2)
~/me/spbu/rust/bigfilestat $ ./target/release/bigfilestat
    LEN             100_000_000                                                             (5.617566125s)
    MIN, MAX        (3.819822494817282e-8, 38.67522961537808)                               (5.667514542s)
    AVERAGE         2.0001830377823477                                                      (5.641533292s)
    DISPERSION      4.000993007496229                                                       (11.349637875s)
    MEDIAN          1.3862322506737415                                                      (105.734784875s)
    LEFT TAIL       [1.287, 0.302, 4.401, 3.905, 1.648, 0.772, 0.226, 6.471, 1.288, 0.489]  (5.86957925s)
    RIGHT TAIL      [1.638, 3.549, 0.326, 5.515, 3.336, 2.507, 4.015, 0.127, 1.072, 2.031]
    TIME TOOK       139.880641833s
*/