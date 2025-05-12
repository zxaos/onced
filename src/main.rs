use itertools::Itertools;
use std::io::Write;

fn main() {
    let mut in_line = String::with_capacity(8);
    loop {
        print!("> ");
        _ = std::io::stdout().flush();
        in_line.clear();
        match std::io::stdin().read_line(&mut in_line) {
            Ok(0) | Err(_) => {
                println!("done, exiting.");
                break;
            }
            Ok(_) => match InputLine::from(in_line.trim()) {
                InputLine::Text(word) => {
                    let converted_word = word_to_numbers(&word);
                    print!("{} -> {:?} -> ", word, converted_word);
                    if let Some(c) = core(converted_word) {
                        print!("{c}");
                        if let Some(new_c) = number_to_letters(c) {
                            print!(" -> {new_c}");
                        }
                    } else {
                        print!("no valid cores possible");
                    }
                    println!();
                }
                InputLine::OneNum(n) => {
                    print_core(core(split_numstring(n)));
                }
                InputLine::FourNums(n) => {
                    print_core(core(n));
                }
                InputLine::Unknown => println!("Unrecognized input style"),
            },
        }
    }
}

enum InputLine {
    OneNum(usize),
    FourNums([usize; 4]),
    Text(String),
    Unknown,
}

impl From<&str> for InputLine {
    fn from(line: &str) -> Self {
        if let Ok(number) = line.parse::<usize>() {
            return InputLine::OneNum(number);
        }

        if line.chars().count() == 4 && line.chars().all(char::is_alphabetic) {
            return InputLine::Text(line.to_uppercase());
        }

        let numsplit_str: Vec<&str> = line.split(',').collect();
        if numsplit_str.len() == 4 {
            let numsplit_num: Vec<usize> = numsplit_str
                .iter()
                .flat_map(|maybe_num| maybe_num.parse::<usize>())
                .collect();
            if numsplit_num.len() == 4 {
                return InputLine::FourNums(numsplit_num.try_into().expect(
                    "This conversion will not be attempted unless there are four elements",
                ));
            }
        }

        InputLine::Unknown
    }
}

fn split_numstring(input: usize) -> [usize; 4] {
    assert!(input > 999);
    let digits: u32 = input.ilog10() + 1;
    let small_size = digits / 4; // how many digits are in the small sized numbers?
    let large_size = small_size + 1; // The large numbers are only ever one larger than the small ones
    let large_count = digits % 4;
    // The number of small digits is whatever is left after reserving digits for big ones
    // let small_count = (digits - (large_count * large_size)) / small_size;

    // find the split pattern that results in the smallest total
    let base_splits = match large_count {
        0 => [small_size; 4],
        1 => [small_size, small_size, small_size, large_size],
        2 => [small_size, small_size, large_size, large_size],
        3 => [small_size, large_size, large_size, large_size],
        _ => panic!("large count should never be greater than 3"),
    };
    let splits = base_splits
        .into_iter()
        .permutations(base_splits.len())
        .unique();

    let mut lowest_sum = usize::MAX;
    let mut result = [0; 4];

    for split in splits {
        // run each split and see if its sum is lower than the previous one
        let mut next_digit = 0;
        let mut candidate_numbers = [0; 4];

        for number in 0..4 {
            let this_split_digits = split[number];
            for extract in (0..this_split_digits).rev() {
                next_digit += 1;
                let tens_actual = 10usize.pow(extract);
                candidate_numbers[number] += extract_digit(input, next_digit) * tens_actual;
            }
        }
        let sum = candidate_numbers.iter().sum();
        if sum < lowest_sum {
            lowest_sum = sum;
            result = candidate_numbers
        }
    }

    result
}

fn extract_digit(source: usize, digit: u32) -> usize {
    assert!(digit > 0);
    let offset = match source {
        0 => 0,
        s => s.ilog10() + 1 - digit,
    };
    (source / 10usize.pow(offset)) % 10
}

fn word_to_numbers(word: &str) -> [usize; 4] {
    assert_eq!(word.chars().count(), 4);
    word.chars()
        .map(|c| (c as u8 - b'A' + 1) as usize)
        .collect_array()
        .expect("failed to convert word")
}

fn number_to_letters(num: usize) -> Option<char> {
    if num > 0 && num < 27 {
        Some((b'A' + (num as u8) - 1) as char)
    } else {
        None
    }
}

// Could probably newtype this to implement the math functions as ops instead
type OptU = Option<usize>;

fn print_core(input: OptU) {
    if let Some(corenum) = input {
        println!("{corenum}");
    } else {
        println!("No valid cores possible");
    }
}

type OptUOp = fn(usize, OptU) -> OptU;

fn core(input: [usize; 4]) -> OptU {
    // Unroll all the possible patterns
    const PATTERNS: [[(OptUOp, usize); 3]; 6] = [
        /* unfortunately, we must accept fractional parts during the calculation
        so long as the end result is whole. So any time there's adjacent div and
        mult operations, run mult first.
        i.e. swap the operands, not the operators.
        */
        [(s_sub, 1), (s_mul, 2), (s_div, 3)],
        [(s_sub, 1), (s_mul, 3), (s_div, 2)], // Swapped
        [(s_mul, 1), (s_sub, 2), (s_div, 3)],
        [(s_mul, 1), (s_div, 2), (s_sub, 3)],
        [(s_div, 1), (s_sub, 2), (s_mul, 3)],
        [(s_mul, 2), (s_div, 1), (s_sub, 3)], // Swapped
    ];

    // For each pattern, apply all the operations with the operand at the
    // supplied index, then return the smallest overall result
    PATTERNS
        .iter()
        .flat_map(|pattern| {
            pattern
                .iter()
                .try_fold(input[0], |acc, (operation, operand)| {
                    operation(acc, Some(input[*operand]))
                })
        })
        .min()
}

fn s_sub(a: usize, b: OptU) -> OptU {
    // If we would go negative, we can't possibly return a whole number later
    a.checked_sub(b?)
}

fn s_mul(a: usize, b: OptU) -> OptU {
    Some(a * b?)
}

fn s_div(a: usize, b: OptU) -> OptU {
    match b {
        Some(b) if b > 0 && a % b == 0 => Some(a / b),
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_numstring() {
        assert_eq!(split_numstring(86455), [8, 6, 45, 5]);
        assert_eq!(split_numstring(3614), [3, 6, 1, 4]);
    }

    #[test]
    fn test_extract_digit() {
        assert_eq!(extract_digit(0, 1), 0, "0@1 = 0");
        assert_eq!(extract_digit(1, 1), 1, "1@1 = 1");
        assert_eq!(extract_digit(10, 1), 1, "10@1 = 1");
        assert_eq!(extract_digit(10, 2), 0, "10@2 = 0");
        assert_eq!(extract_digit(123456, 1), 1, "123456@1 = 1");
        assert_eq!(extract_digit(123456, 2), 2, "123456@1 = 2");
        assert_eq!(extract_digit(123456, 3), 3, "123456@3 = 3");
    }

    #[test]
    fn test_sub() {
        assert_eq!(s_sub(3, Some(1)), Some(2));
        assert_eq!(s_sub(3, None), None);
    }

    #[test]
    fn test_div() {
        assert_eq!(s_div(4, Some(2)), Some(2));
        assert_eq!(s_div(15, Some(5)), Some(3));
        assert_eq!(s_div(15, Some(4)), None);
        assert_eq!(s_div(15, Some(0)), None);
    }

    #[test]
    fn test_core() {
        assert_eq!(core([8, 6, 45, 5]), Some(18));
        assert_eq!(core([1000, 200, 11, 2]), Some(53));
        assert_eq!(core([8, 1, 14, 4]), Some(2));
    }
}
